use std::sync::{Arc, Condvar, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;


struct Actor<T, I, O> {
    state: Arc<Mutex<T>>,
    rx: Option<Receiver<I>>,
    tx: Sender<O>,
    condition: Arc<(Mutex<bool>, Condvar)>,
}


impl<T, I, O> Clone for Actor<T, I, O> {
    fn clone(&self) -> Self {
        Actor {
            state: self.state.clone(),
            rx: None,
            tx: self.tx.clone(),
            condition: self.condition.clone(),
        }
    }
}

struct ActorProxy<I, O> {
    tx: Sender<I>,
    rx: Receiver<O>,
}


impl<T, I, O> Actor<T, I, O> {
    fn new<R>(data: T, receive_callback: R, others: Vec<Box<dyn Fn(Actor<T, I, O>) -> anyhow::Result<()> + Send + Sync + 'static>>) -> ActorProxy<I, O>
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
            };
            actor.run(receive_callback, others)
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
            let result = receive_callback(self);

            result
        }));

        for handle in handles {
            // handle.thread().unpark();
            handle.join().unwrap().unwrap();
        }
        Ok(())
    }

    fn notify_all(&self) {
        let (lock, cvar) = &*self.condition;
        let foo = lock.lock().unwrap();

        cvar.notify_all();
    }

    fn park(&self) {
        let (lock, cvar) = &*self.condition;
        let foo = lock.lock().unwrap();

        let _guard = cvar.wait(foo).unwrap();
    }

    fn join(&self) {
        if let Some(rx) = &self.rx {
            while let Ok(message) = rx.recv() {
                // discard messages
            }
        } else {
            // if !self.main.is_finished() {
            //     self.main.join()
            // }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::actor::{Actor, ActorProxy};

    struct State {
        count: u64,
    }

    struct BusinessLogic {}

    struct MessageIn {}

    struct MessageOut {
        count: u64,
    }

    fn receiver(actor: Actor<State, MessageIn, MessageOut>) -> anyhow::Result<()> {
        if let Some(rx) = &actor.rx {
            while let Ok(message) = rx.recv() {
                let mut state = actor.state.lock().unwrap();
                (*state).count += 1;
            }
        }


        println!("Done receiving");
        actor.notify_all();

        Ok(())
    }

    fn idler(actor: Actor<State, MessageIn, MessageOut>) -> anyhow::Result<()> {
        {
            let mut state = actor.state.lock().unwrap();
            (*state).count += 1000;
        }

        actor.park();
        //thread::sleep(Duration::from_secs(1));


        let count = {
            let mut state = actor.state.lock().unwrap();
            (*state).count
        };

        actor.tx.send(MessageOut { count })?;
        Ok(())
    }

    #[test]
    fn test_it() {
        let actor: ActorProxy<MessageIn, MessageOut> = Actor::new(
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
