pub mod main_worker_host;
pub mod ops;



deno_core::extension!(deno_mf);
  

  
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum MfError {
  #[class(inherit)]
  #[error(transparent)]
  Transaction(#[from] deno_core::error::ResourceError),
  
  #[class(inherit)]
  #[error(transparent)]
  Other(deno_error::JsErrorBox),
}