use anyhow::Result;

pub struct ApprovalRequest<'a> {
    pub call_id: &'a str,
    pub tool_name: &'a str,
    pub arguments: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
}

pub trait ApprovalHandler {
    fn request_approval(&mut self, request: &ApprovalRequest<'_>) -> Result<ApprovalDecision>;
}
