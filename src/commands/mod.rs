pub mod clone;
pub use clone::Clone;

pub mod install;
pub(crate) use install::Install;

pub mod link;
pub(crate) use link::Link;

pub mod init;
pub use init::Init;

pub trait Command {
  type Args;
  type Result;

  fn execute(&self, args: Self::Args) -> Self::Result;
}
