use std::sync::{Arc, Condvar, mpsc, Mutex};
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::thread;


pub struct Actor<T, I, O> {
    pub state: Arc<Mutex<T>>,
    pub rx: Option<Receiver<I>>,
    pub tx: Sender<O>,
    condition: Arc<(Mutex<bool>, Condvar)>,
    stopped: Arc<Mutex<bool>>,
}


impl<T, I, O> Clone for Actor<T, I, O> {
    fn clone(&self) -> Self {
        Actor {
            state: self.state.clone(),
            rx: None,
            tx: self.tx.clone(),
            condition: self.condition.clone(),
            stopped: self.stopped.clone(),
        }
    }
}

pub struct ActorProxy<I, O> {
    pub tx: Sender<I>,
    pub rx: Receiver<O>,
}


impl<T, I, O> Actor<T, I, O> {
    pub fn new<R1, R2>(data: T, primary: R1, secondary: R2) -> ActorProxy<I, O>
        where
            R1: Fn(Actor<T, I, O>) -> anyhow::Result<()> + Send + Sync + 'static,
            R2: Fn(Actor<T, I, O>) -> anyhow::Result<()> + Send + Sync + 'static,
            T: Send + 'static,
            I: Send + 'static,
            O: Send + 'static,
    {
        Actor::new_actor(
            data,
            primary,
            vec![
                Box::new(secondary)
            ])
    }
    pub fn new_actor<R>(data: T, primary: R, secondaries: Vec<Box<dyn Fn(Actor<T, I, O>) -> anyhow::Result<()> + Send + Sync + 'static>>) -> ActorProxy<I, O>
        where
            R: Fn(Actor<T, I, O>) -> anyhow::Result<()> + Send + Sync + 'static,
            T: Send + 'static,
            I: Send + 'static,
            O: Send + 'static,
    {
        let (input_tx, input_rx) = mpsc::channel();
        let (output_tx, output_rx) = mpsc::channel();

        let proxy = ActorProxy {
            tx: input_tx,
            rx: output_rx,
        };


        thread::spawn(move || {
            let actor = Actor {
                state: Arc::new(Mutex::new(data)),
                rx: Some(input_rx),
                tx: output_tx,
                condition: Arc::new((Mutex::new(true), Condvar::new())),
                stopped: Arc::new(Mutex::new(false)),
            };
            actor.run(primary, secondaries)
        });

        proxy
    }

    fn run<R>(mut self, receive_callback: R, others: Vec<Box<dyn Fn(Actor<T, I, O>) -> anyhow::Result<()> + Send + Sync + 'static>>) -> anyhow::Result<()>
        where
            R: Fn(Actor<T, I, O>) -> anyhow::Result<()> + Send + Sync + 'static,
            T: Send + 'static,
            I: Send + 'static,
            O: Send + 'static,
    {
        let mut handles = vec![];
        for other in others {
            let actor = self.clone();
            handles.push(thread::spawn(move || {
                other(actor)
            }));
        }

        handles.push(thread::spawn(move || {
            let mut sclone = self.clone();
            let result = receive_callback(self);
            sclone.stop();
            sclone.notify_all();
            result
        }));

        for handle in handles {
            handle.join().unwrap().unwrap();
        }
        Ok(())
    }

    pub fn notify_all(&self) {
        let (lock, cvar) = &*self.condition;
        let foo = lock.lock().unwrap();

        cvar.notify_all();
    }

    pub fn park(&self) {
        let (lock, cvar) = &*self.condition;
        let foo = lock.lock().unwrap();

        let _guard = cvar.wait(foo).unwrap();
    }

    pub fn receive(&self) -> anyhow::Result<I> {
        if let Some(rx) = &self.rx {
            Ok(rx.recv()?)
        } else {
            Err(anyhow::anyhow!("Only the primary can receive messages!"))
        }
    }

    pub fn is_stopped(&self) -> bool {
        if let Ok(stopped) = self.stopped.lock() {
            *stopped
        } else {
            true
        }
    }

    pub fn stop(&mut self) {
        if let Ok(mut stopped) = self.stopped.lock() {
            *stopped = true
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::actor::{Actor, ActorProxy};

    struct State {
        count: u64,
    }


    struct MessageIn {}

    struct MessageOut {
        count: u64,
    }

    fn receiver(actor: Actor<State, MessageIn, MessageOut>) -> anyhow::Result<()> {
        if let Some(rx) = &actor.rx {
            while let Ok(message) = rx.recv() {
                {
                    let mut state = actor.state.lock().unwrap();
                    (*state).count += 1;
                }
                actor.notify_all();
            }
        }

        println!("Done receiving");
        Ok(())
    }

    fn idler(actor: Actor<State, MessageIn, MessageOut>) -> anyhow::Result<()> {
        {
            let mut state = actor.state.lock().unwrap();
            (*state).count += 1000;
        }

        actor.park();

        let count = {
            let mut state = actor.state.lock().unwrap();
            (*state).count
        };

        actor.tx.send(MessageOut { count })?;
        Ok(())
    }

    #[test]
    fn test_it() {
        let actor: ActorProxy<MessageIn, MessageOut> = Actor::new_actor(
            State {
                count: 0,
            },
            move |actor| receiver(actor),
            vec![
                Box::new(|actor| idler(actor)),
            ],
        );

        let tx = actor.tx;
        for i in 0..10 {
            tx.send(MessageIn {}).unwrap();
        }
        drop(tx);

        println!("Dropped tx");

        let result = actor.rx.recv().unwrap();
        println!("Written and read! {}", result.count);
        assert_eq!(result.count, 1010);
    }
}
