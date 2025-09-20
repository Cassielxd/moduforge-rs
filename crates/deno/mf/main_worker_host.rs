// Copyright 2018-2025 the Deno authors. MIT license.

use std::rc::Rc;
use std::sync::Arc;
use dashmap::DashMap;
use deno_core::CancelHandle;
use deno_core::Extension;
use deno_core::FsModuleLoader;
use deno_core::error::CoreError;
use deno_fs::RealFs;
use deno_permissions::Permissions;
use deno_permissions::PermissionsContainer;
use deno_resolver::npm::DenoInNpmPackageChecker;
use log::debug;
use url::Url;

use crate::permissions::RuntimePermissionDescriptorParser;
use crate::tokio_util::create_and_run_current_thread;

use crate::worker::MainWorkerInternalHandle;

use crate::worker::MainWorker;
use crate::worker::MainWorkerHandle;
use crate::worker::MainWorkerId;
use crate::worker::WorkerOptions;
use crate::worker::WorkerServiceOptions;
pub type CliSys = sys_traits::impls::RealSys;
pub type CliNpmResolver = deno_resolver::npm::NpmResolver<CliSys>;


pub struct MainWorkerThread {
  worker_handle: MainWorkerHandle,
  cancel_handle: Rc<CancelHandle>,
}

impl MainWorkerThread {
  fn terminate(self) {
    self.cancel_handle.cancel();
  }
}

impl Drop for MainWorkerThread {
  fn drop(&mut self) {
    self.worker_handle.clone().terminate();
  }
}

pub type MainWorkersTable = DashMap<MainWorkerId, MainWorkerThread>;

pub struct MainWorkerFactory {
  table: MainWorkersTable,
}

impl MainWorkerFactory {
  pub fn new() -> Self {
    Self {
      table: MainWorkersTable::new(),
    }
  }
  pub fn worker_exists(&self,id: MainWorkerId)->bool{
    self.table.contains_key(&id)
  }

  pub fn dispatch(&self,id: MainWorkerId){
    if let Some(worker) = self.table.get_mut(&id){
      
    }
  }
  pub fn worker_close(&mut self,id: MainWorkerId){
      match self.table.remove(&id) {
      Some((_,worker_thread)) => {
        worker_thread.terminate();
      }
      _ => {
        debug!("tried to terminate non-existent worker {}", id);
      }
    }
  }

  #[allow(clippy::result_large_err)]
  pub fn create_main_worker(
    &mut self,
    name:String,
    main_module: Url,
  ) -> Result<MainWorkerId, CoreError> {
    let worker_id = MainWorkerId::new();
     let thread_builder = std::thread::Builder::new().name(format!("main_{worker_id}"));
       let (handle_sender, handle_receiver) =
    std::sync::mpsc::sync_channel::<MainWorkerInternalHandle>(1);
   thread_builder.spawn(move || {
    
    let fut = async move {
      let (mut worker, external_handle) =
        MainWorkerFactory::create_custom_worker(
          worker_id,
      name,
      main_module,
      vec![],
    ).unwrap();
      let exit_code = worker.run().await;
      handle_sender.send(external_handle).unwrap();
      drop(handle_sender);

     
    };

    create_and_run_current_thread(fut)
  })?;
  let worker_handle = handle_receiver.recv().unwrap();
  let worker_thread = MainWorkerThread {
    worker_handle: worker_handle.into(),
    cancel_handle: CancelHandle::new_rc(),
  };
  self.table.insert(worker_id, worker_thread);
  Ok(worker_id)
    
  }

  #[allow(clippy::result_large_err)]
  #[allow(clippy::too_many_arguments)]
  pub fn create_custom_worker(
    worker_id: MainWorkerId,
    name:String,
    main_module: Url,
    custom_extensions: Vec<Extension>
  ) -> Result<(MainWorker,MainWorkerInternalHandle), CoreError> {
    let fs = Arc::new(RealFs);
    let permission_desc_parser: Arc<RuntimePermissionDescriptorParser<sys_traits::impls::RealSys>> = Arc::new(
      RuntimePermissionDescriptorParser::new(sys_traits::impls::RealSys::default()),
    );
    let services = WorkerServiceOptions {
        deno_rt_native_addon_loader: None,
        module_loader: Rc::new(FsModuleLoader),
        permissions: PermissionsContainer::new(
          permission_desc_parser,
          Permissions::allow_all(),
        ),
        blob_store: Default::default(),
        broadcast_channel: Default::default(),
        feature_checker: Default::default(),
        node_services: Default::default(),
        npm_process_state_provider: Default::default(),
        root_cert_store_provider: Default::default(),
        fetch_dns_resolver: Default::default(),
        shared_array_buffer_store: Default::default(),
        compiled_wasm_module_store: Default::default(),
        v8_code_cache: Default::default(),
        fs,
        bundle_provider: None,
      };

    let options = WorkerOptions{
      worker_id,
      name,
      extensions: custom_extensions,
      ..Default::default()
    };
   let result =MainWorker::bootstrap_from_options::<
      DenoInNpmPackageChecker,
      CliNpmResolver,
      CliSys,
    >(&main_module, services, options);
    Ok(result)
  }

}
