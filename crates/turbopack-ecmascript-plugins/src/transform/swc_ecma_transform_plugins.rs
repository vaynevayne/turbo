use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use async_trait::async_trait;
use swc_core::{
    common::{comments::SingleThreadedComments, util::take::Take, FileName},
    ecma::{
        ast::{Module, Program},
        visit::FoldWith,
    },
    plugin_runner::plugin_module_bytes::CompiledPluginModuleBytes,
};
use turbo_tasks_fs::File;
use turbopack_ecmascript::{CustomTransformer, TransformContext};

#[turbo_tasks::value(transparent)]
pub struct PluginModule(CompiledPluginModuleBytes);

#[derive(Debug)]
pub struct SwcEcmaTransformPluginsTransformer {
    #[cfg_attr(not(feature = "swc_ecma_transform_plugin"), allow(unused))]
    plugins: Vec<(PluginModuleVc, serde_json::Value)>,
}

impl SwcEcmaTransformPluginsTransformer {
    pub fn new(plugins: Vec<(PluginModuleVc, serde_json::Value)>) -> Self {
        Self { plugins }
    }
}

#[async_trait]
impl CustomTransformer for SwcEcmaTransformPluginsTransformer {
    #[cfg_attr(not(feature = "swc_ecma_transform_plugin"), allow(unused))]
    async fn transform(&self, program: &mut Program, ctx: &TransformContext<'_>) -> Result<()> {
        #[cfg(feature = "swc_ecma_transform_plugin")]
        {
            use std::{path::PathBuf, sync::Arc};

            use anyhow::Context;
            use swc_core::{
                common::plugin::{
                    metadata::TransformPluginMetadataContext, serialized::PluginSerializedBytes,
                },
                plugin::proxies::{HostCommentsStorage, COMMENTS},
                plugin_runner::cache::PLUGIN_MODULE_CACHE,
            };

            //[TODO]: as same as swc/core does, we should set should_enable_comments_proxy
            // depends on the src's comments availability. For now, check naively if leading
            // / trailing comments are empty.
            let should_enable_comments_proxy =
                !ctx.comments.leading.is_empty() && !ctx.comments.trailing.is_empty();

            let comments = if should_enable_comments_proxy {
                // Plugin only able to accept singlethreaded comments, interop from
                // multithreaded comments.
                let mut leading =
                    swc_core::common::comments::SingleThreadedCommentsMapInner::default();
                ctx.comments.leading.as_ref().into_iter().for_each(|c| {
                    leading.insert(c.key().clone(), c.value().clone());
                });

                let mut trailing =
                    swc_core::common::comments::SingleThreadedCommentsMapInner::default();
                ctx.comments.trailing.as_ref().into_iter().for_each(|c| {
                    trailing.insert(c.key().clone(), c.value().clone());
                });

                Some(SingleThreadedComments::from_leading_and_trailing(
                    Rc::new(RefCell::new(leading)),
                    Rc::new(RefCell::new(trailing)),
                ))
            } else {
                None
            };

            let mut plugins = vec![];
            for (plugin_module, config) in &self.plugins {
                let plugin_module = plugin_module.await?;
                plugins.push(
                    plugin_module.get_name().clone(),
                    plugin_module.compile_bytes()?,
                );
            }

            let transformed_program: Program =
                COMMENTS.set(&HostCommentsStorage { inner: comments }, || {
                    let mut serialized_program = PluginSerializedBytes::try_serialize(program)?;

                    // Run plugin transformation against current program.
                    // We do not serialize / deserialize between each plugin execution but
                    // copies raw transformed bytes directly into plugin's memory space.
                    // Note: This doesn't mean plugin won't perform any se/deserialization: it
                    // still have to construct from raw bytes internally to perform actual
                    // transform.
                    for (plugin_module, config) in &self.plugins {
                        let plugin_module = plugin_module.await?;

                        let transform_metadata_context =
                            Arc::new(TransformPluginMetadataContext::new(
                                Some(ctx.file_name_str.to_string()),
                                //[TODO]: Support env-related variable injection, i.e process.env.NODE_ENV
                                "development".to_string(),
                                None,
                            ));

                        let mut transform_plugin_executor =
                            swc_plugin_runner::create_plugin_transform_executor(
                                &PathBuf::from(name),
                                &PLUGIN_MODULE_CACHE,
                                ctx.source_map,
                                &transform_metadata_context,
                                Some(config.clone()),
                            )?;

                        if !transform_plugin_executor.is_transform_schema_compatible()? {
                            anyhow::bail!("Cannot execute incompatible plugin {}", name);
                        }

                        serialized_program = transform_plugin_executor
                            .transform(
                                &serialized_program,
                                ctx.unresolved_mark,
                                should_enable_comments_proxy,
                            )
                            .with_context(|| {
                                format!(
                                    "failed to invoke `{}` as js transform plugin at {}",
                                    name, ctx.file_name_str
                                )
                            })?;
                    }

                    // Plugin transformation is done. Deserialize transformed bytes back
                    // into Program
                    serialized_program.deserialize()
                })?;

            *program = transformed_program;
        }

        Ok(())
    }
}
