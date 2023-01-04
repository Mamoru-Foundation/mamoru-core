/// Required to sign a transaction for each message type
use crate::validation_chain::proto::validation_chain::{
    MsgRegisterRule, MsgRegisterSniffer, MsgReportIncident, MsgSubscribeRules, MsgUnregisterSniffer,
};
use cosmrs::proto::traits::TypeUrl;

impl TypeUrl for MsgRegisterSniffer {
    const TYPE_URL: &'static str = "/validationchain.validationchain.MsgRegisterSniffer";
}

impl TypeUrl for MsgUnregisterSniffer {
    const TYPE_URL: &'static str = "/validationchain.validationchain.MsgUnregisterSniffer";
}

impl TypeUrl for MsgSubscribeRules {
    const TYPE_URL: &'static str = "/validationchain.validationchain.MsgSubscribeRules";
}

impl TypeUrl for MsgReportIncident {
    const TYPE_URL: &'static str = "/validationchain.validationchain.MsgReportIncident";
}

impl TypeUrl for MsgRegisterRule {
    const TYPE_URL: &'static str = "/validationchain.validationchain.MsgRegisterRule";
}
