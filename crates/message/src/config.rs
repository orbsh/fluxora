use serde::Deserialize;

#[cfg(feature = "kafka")]
#[derive(Debug, Deserialize, Clone)]
pub struct KafkaIncomeConfig {
    pub broker: Vec<String>,
    pub topic: Vec<String>,
    pub group: Option<String>,
}

#[cfg(feature = "kafka")]
#[derive(Debug, Deserialize, Clone)]
pub struct KafkaOutgoConfig {
    pub broker: Vec<String>,
    pub topic: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[allow(unused)]
pub enum QueueIncome {
    #[cfg(feature = "kafka")]
    #[allow(non_camel_case_types)]
    kafka(KafkaIncomeConfig),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[allow(unused)]
pub enum QueueOutgo {
    #[cfg(feature = "kafka")]
    #[allow(non_camel_case_types)]
    kafka(KafkaOutgoConfig),
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Queue {
    #[serde(default)]
    pub disable: bool,
    pub outgo: QueueOutgo,
    pub income: QueueIncome,
}
