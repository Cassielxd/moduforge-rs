use std::sync::Arc;

use mf_core::{
    extension::Extension,
    types::{Content, EditorOptionsBuilder, Extensions, NodePoolFnTrait},
};
use mf_state::plugin::{Plugin, PluginSpec};

use crate::{
    core::{
        collab_editor::{CollabEditor, CollabEditorOptions},
        demo_editor::{DemoEditor, DemoEditorOptions},
    },
    marks, middleware,
    nodes::{
        fbfx_csxm::{init_fbfx_csxm_fields, CSXM_STR, DE_STR, FBFX_STR},
        gcxm::{init_project_structure, DWGC_STR},
        rcj::{init_rcj_fields, RCJ_STR},
    },
    plugins::inc::{IncStateField, IncStatePlugin},
};
//获取编辑器
pub async fn init_editor(options: DemoEditorOptions) -> DemoEditor {
    DemoEditor::create(options).await.unwrap()
}

//获取编辑器
pub async fn init_collab_editor(options: CollabEditorOptions) -> CollabEditor {
    match CollabEditor::create(options).await {
        Ok(editor) => editor,
        Err(e) => {
            println!("创建编辑器失败: {e}");
            panic!("创建编辑器失败: {e}");
        },
    }
}

//获取编辑器配置
pub async fn init_collab_options(
    create_callback: Arc<dyn NodePoolFnTrait>,
    room_name: String,
) -> CollabEditorOptions {
    let mut builder = EditorOptionsBuilder::new();
    builder = builder
        .content(Content::NodePoolFn(create_callback))
        // 设置历史记录限制
        .history_limit(20)
        // 添加扩展
        .extensions(init_extension())
        // 添加中间件
        .add_middleware(
            middleware::collect_fbfx_csxm::CollectFbfxCsxmMiddleware,
        );
    let options = builder.build();
    CollabEditorOptions {
        editor_options: options,
        server_url: "ws://127.0.0.1:8080/collaboration".to_string(),
        room_name,
    }
}

//获取编辑器配置
pub async fn init_options(
    create_callback: Arc<dyn NodePoolFnTrait>
) -> DemoEditorOptions {
    let mut builder = EditorOptionsBuilder::new();
    builder = builder
        .content(Content::NodePoolFn(create_callback))
        // 设置历史记录限制
        .history_limit(20)
        // 添加扩展
        .extensions(init_extension())
        // 添加中间件
        .add_middleware(
            middleware::collect_fbfx_csxm::CollectFbfxCsxmMiddleware,
        );
    let options = builder.build();
    DemoEditorOptions { editor_options: options }
}
//获取扩展
pub fn init_extension() -> Vec<Extensions> {
    let mut extensions = vec![
        Extensions::M(marks::BG_COLOR.clone()),
        Extensions::M(marks::FOOTNOTE.clone()),
    ];
    // 工程项目、单项、单位Node
    let nodes = init_project_structure();
    for mut node in nodes {
        if node.get_name() == DWGC_STR {
            node.set_content(&format!("{FBFX_STR}|{CSXM_STR}+"));
        }
        extensions.push(Extensions::N(node));
    }
    let fbfx_csxm_nodes = init_fbfx_csxm_fields();
    for mut node in fbfx_csxm_nodes {
        if node.get_name() == DE_STR {
            node.set_content(&format!("{RCJ_STR}*"));
        }
        extensions.push(Extensions::N(node));
    }
    // 定额下人材机明细Node
    let rcj_node = init_rcj_fields();
    extensions.push(Extensions::N(rcj_node));
    let mut extension = Extension::new();
    let inc_plugin = Plugin::new(PluginSpec {
        state_field: Some(Arc::new(IncStateField)),
        tr: Arc::new(IncStatePlugin),
    });
    extension.add_plugin(Arc::new(inc_plugin));
    extensions.push(Extensions::E(extension));
    extensions
}
