//! `ServerHandler` on `QdoMcp`: capabilities, the `qdo36://state`
//! resource and its subscribe/unsubscribe.

use rmcp::ErrorData;
use rmcp::RoleServer;
use rmcp::ServerHandler;
use rmcp::model::ListResourcesResult;
use rmcp::model::PaginatedRequestParams;
use rmcp::model::ReadResourceRequestParams;
use rmcp::model::ReadResourceResponse;
use rmcp::model::ReadResourceResult;
use rmcp::model::Resource;
use rmcp::model::ResourceContents;
use rmcp::model::ServerCapabilities;
use rmcp::model::ServerConfig;
use rmcp::model::SubscribeRequestParams;
use rmcp::model::UnsubscribeRequestParams;
use rmcp::service::RequestContext;
use rmcp::tool_handler;

use crate::McpError;
use crate::STATE_URI;
use crate::qdo_mcp::QdoMcp;

#[tool_handler]
impl ServerHandler for QdoMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_resources().enable_resources_subscribe().build()).with_instructions(
            "Quai des Orfevres 36 - Scotland Yard on the tangible-objects \
             surface. The qdo36://state resource mirrors the app's live \
             snapshot (anchor pose, map) and supports resources/subscribe \
             for change notifications. Play/introspection tools land in \
             later phases - the schema stays additive.",
        )
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, ErrorData>> {
        std::future::ready(Ok(ListResourcesResult::with_all_items(vec![
            Resource::new(STATE_URI, "state")
                .with_title("Live game state")
                .with_description("Current app snapshot: anchor detection and pose, device surface size.")
                .with_mime_type("application/json"),
        ])))
    }

    #[allow(clippy::manual_async_fn)]
    async fn read_resource(&self, request: ReadResourceRequestParams, _context: RequestContext<RoleServer>) -> Result<ReadResourceResponse, ErrorData> {
        if request.uri != STATE_URI {
            return Err(McpError::InvalidParams(format!("unknown resource {}", request.uri)).into());
        }
        let text = serde_json::to_string_pretty(&self.snapshot()).map_err(McpError::from)?;
        Ok(ReadResourceResult::new(vec![ResourceContents::text(text, STATE_URI).with_mime_type("application/json")]).into())
    }

    #[allow(deprecated)]
    fn subscribe(&self, request: SubscribeRequestParams, context: RequestContext<RoleServer>) -> impl Future<Output = Result<(), ErrorData>> {
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers.push((request.uri, context.peer.clone()));
        }
        std::future::ready(Ok(()))
    }

    #[allow(deprecated)]
    fn unsubscribe(&self, request: UnsubscribeRequestParams, _context: RequestContext<RoleServer>) -> impl Future<Output = Result<(), ErrorData>> {
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers.retain(|(uri, _)| uri != &request.uri);
        }
        std::future::ready(Ok(()))
    }
}
