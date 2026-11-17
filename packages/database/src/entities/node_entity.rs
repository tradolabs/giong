use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct NodeEntity {
    pub id: Uuid,
    pub flow_id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub properties: serde_json::Value,
    pub triggers: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}
