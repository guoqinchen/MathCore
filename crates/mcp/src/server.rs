//! MCP Server implementation

use crate::protocol::{McpErrorCode, McpRequest, McpResponse, McpTool};
use crate::tools::MathCoreTools;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct McpServer {
    tools: Arc<RwLock<Vec<McpTool>>>,
    math_tools: Arc<MathCoreTools>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(Vec::new())),
            math_tools: Arc::new(MathCoreTools::new()),
        }
    }

    pub async fn register_tool(&self, tool: McpTool) {
        let mut tools = self.tools.write().await;
        tools.push(tool);
    }

    pub async fn list_tools(&self) -> Vec<McpTool> {
        let tools = self.tools.read().await;
        tools.clone()
    }

    pub async fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "tools/list" => {
                let tools = self.list_tools().await;
                McpResponse::success(
                    request.id,
                    serde_json::json!({
                        "tools": tools
                    }),
                )
            }
            "tools/call" => {
                if let Some(params) = request.params {
                    if let (Some(tool_name), Some(args)) = (
                        params.get("name").and_then(|v| v.as_str()),
                        params.get("arguments").and_then(|v| v.as_object()),
                    ) {
                        match self.call_tool(tool_name, args.clone()).await {
                            Ok(result) => McpResponse::success(request.id, result),
                            Err(e) => McpResponse::error(
                                request.id,
                                McpErrorCode::InternalError as i32,
                                e.to_string(),
                            ),
                        }
                    } else {
                        McpResponse::error(
                            request.id,
                            McpErrorCode::InvalidParams as i32,
                            "Missing tool name or arguments",
                        )
                    }
                } else {
                    McpResponse::error(
                        request.id,
                        McpErrorCode::InvalidParams as i32,
                        "Missing params",
                    )
                }
            }
            "initialize" => McpResponse::success(
                request.id,
                serde_json::json!({
                    "protocolVersion": "1.0",
                    "serverInfo": {
                        "name": "mathcore-mcp",
                        "version": "0.6.0"
                    },
                    "capabilities": {
                        "tools": {}
                    }
                }),
            ),
            _ => McpResponse::error(
                request.id,
                McpErrorCode::MethodNotFound as i32,
                format!("Unknown method: {}", request.method),
            ),
        }
    }

    async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        match name {
            "evaluate" => {
                let expr = args
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("x^2");
                let result = self.math_tools.evaluate(expr).await;
                Ok(serde_json::json!({ "result": result }))
            }
            "simplify" => {
                let expr = args
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("x + 0");
                let result = self.math_tools.simplify(expr).await;
                Ok(serde_json::json!({ "result": result }))
            }
            "derivative" => {
                let expr = args
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("x^2");
                let var = args.get("variable").and_then(|v| v.as_str()).unwrap_or("x");
                let result = self.math_tools.derivative(expr, var).await;
                Ok(serde_json::json!({ "result": result }))
            }
            "integrate" => {
                let expr = args
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("x^2");
                let var = args.get("variable").and_then(|v| v.as_str()).unwrap_or("x");
                let result = self.math_tools.integrate(expr, var).await;
                Ok(serde_json::json!({ "result": result }))
            }
            _ => Err(format!("Unknown tool: {}", name).into()),
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
