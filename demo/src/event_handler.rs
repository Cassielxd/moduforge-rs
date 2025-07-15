

use mf_core::{event::EventHandler, Event, ForgeResult};
use mf_state::{state::State, transaction::Transaction};

#[derive(Debug)]
pub struct DemoEventHandler;

#[async_trait::async_trait]
impl EventHandler<Event> for DemoEventHandler {
    async fn handle(&self, event: &Event) -> ForgeResult<()> {
        match event {
            Event::Create(state)=>{println!("🎉 DemoEventHandler: 状态创建: 版本 {}",state.version);Ok(())}
            Event::TrApply(_, transactions, state) => {println!("🎉 DemoEventHandler: 事务应用: 版本 {}",state.version);Ok(())}
            Event::Destroy => {println!("🎉 DemoEventHandler: 状态销毁");Ok(())}
            Event::Stop => {println!("🎉 DemoEventHandler: 停止");Ok(())}
        }
    }
}
