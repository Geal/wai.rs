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

pub trait Middleware {
  type Input: Application;
  type Output: Application;

  fn apply(i: Self::Input) -> Self::Output;
}

#[cfg(test)]
mod tests {
  use super::*;
  use futures::future::{self, FutureResult};

  pub struct MyApp {
    s: String,
    counter: u8,
  }

  pub struct Server<App> {
    app: App,
    counter: u8,
  }

  pub struct LoggingMiddleware<A> {
    app: A,
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
       println!("{}|app got request: {:?}", self.counter, req);
       self.counter += 1;
       respond(Response)
    }
  }

  impl<A:Application> Middleware for LoggingMiddleware<A> {
    type Input = A;
    type Output = Self;
    fn apply(a: A) -> LoggingMiddleware<A> {
      LoggingMiddleware { app: a }
    }
  }

  impl<A: Application> Application for LoggingMiddleware<A> {
    type Output = <A as Application>::Output;
    fn run<F>(&mut self, req: Request, respond: F) -> Self::Output
      where F: Fn(Response) -> Self::Output {
      println!("logging");
      self.app.run(req, respond)
    }
  }


  #[test]
  fn test() {
    let mut server = Server::new(
      LoggingMiddleware::apply(MyApp { s: "app".to_string(), counter: 0 })
    );
    server.run();
  }
}
