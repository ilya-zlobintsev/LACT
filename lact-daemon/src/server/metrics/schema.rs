use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsPayload<'a> {
    pub resource_metrics: Vec<ResourceMetric<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetric<'a> {
    pub resource: Resource<'a>,
    pub scope_metrics: Vec<ScopeMetric<'a>>,
}

#[derive(Serialize)]
pub struct Resource<'a> {
    pub attributes: Vec<Attribute<'a>>,
}

#[derive(Serialize)]
pub struct ScopeMetric<'a> {
    pub scope: Scope<'a>,
    pub metrics: Vec<Metric<'a>>,
}

#[derive(Serialize)]
pub struct Scope<'a> {
    pub name: String,
    pub version: String,
    pub attributes: Vec<Attribute<'a>>,
}

#[derive(Serialize, Clone)]
pub struct Attribute<'a> {
    pub key: &'static str,
    pub value: Value<'a>,
}

#[derive(Serialize)]
pub struct Metric<'a> {
    pub name: &'static str,
    pub unit: &'static str,
    pub description: &'static str,
    pub gauge: Gauge<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gauge<'a> {
    pub data_points: Vec<GaugeDataPoint<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeDataPoint<'a> {
    #[serde(flatten)]
    pub value: NumberValue,
    pub time_unix_nano: &'a str,
    pub attributes: Vec<Attribute<'a>>,
}

#[derive(Serialize)]
pub enum NumberValue {
    #[serde(rename = "asDouble")]
    Float(f64),
    #[serde(rename = "asInt")]
    Int(i64),
}

impl From<f64> for NumberValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<i64> for NumberValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

#[derive(Serialize, Clone)]
pub enum Value<'a> {
    #[serde(rename = "stringValue")]
    String(&'a str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_value() {
        let value = Value::String("asd");
        let expected_result = r#"{"stringValue":"asd"}"#;
        assert_eq!(expected_result, serde_json::to_string(&value).unwrap());
    }
}
