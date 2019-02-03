//! type Application = Request -> (Response -> IO ResponseReceived) -> IO ResponseReceived
//#![feature(impl_trait_in_bindings)]

extern crate futures;

use futures::{Future, IntoFuture};

use std::io;

#[derive(Clone,Debug)]
pub struct Request;
#[derive(Clone,Debug)]
pub struct Response;
#[derive(Clone,Debug)]
pub struct ResponseReceived;

pub trait Application {
  type Output: IntoFuture<Item=ResponseReceived, Error=io::Error>;

  fn run<F>(&mut self, req: Request, respond: F) -> Self::Output
    where F: Fn(Response) -> Self::Output;
}

#[cfg(test)]
mod tests {
  use super::*;
  use futures::future::{self, FutureResult};

  pub struct MyApp {
    s: String,
  }

  pub struct Server<App> {
    app: App,
    counter: u8,
  }

  impl<App> Server<App> {
    pub fn new(app: App) -> Self {
      Server { app, counter: 0 }
    }
  }

  impl<App: Application<Output=FutureResult<ResponseReceived, io::Error>>> Server<App> {
    pub fn run(&mut self) {
      for i in 1..3 {
        std::thread::sleep_ms(500);

        let mut received = self.app.run(Request, |response| {
          println!("callback called");
          future::ok(ResponseReceived)
        });

        received.poll().unwrap();
      }
      panic!();
    }
  }

  impl Application for MyApp {
    type Output = FutureResult<ResponseReceived, io::Error>;
    fn run<F>(&mut self, req: Request, respond: F) -> Self::Output
      where F: Fn(Response) -> Self::Output {
       println!("app got request: {:?}", req);
       respond(Response)
    }
  }

  #[test]
  fn test() {
    let mut server = Server::new(MyApp { s: "app".to_string() });
    server.run();
  }
}
