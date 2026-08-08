use crate::domains::communications::extract::{ExtractedNote, ExtractedTask};
use makosh_connectrpc_contracts::makosh::communications::v1::{
    ExtractedNote as ProtoExtractedNote, ExtractedTask as ProtoExtractedTask,
};

pub(super) fn task(item: ExtractedTask) -> ProtoExtractedTask {
    ProtoExtractedTask {
        title: item.title,
        due_date: item.due_date,
        assignee: item.assignee,
        priority: item.priority,
        source: item.source,
        ..Default::default()
    }
}
pub(super) fn note(item: ExtractedNote) -> ProtoExtractedNote {
    ProtoExtractedNote {
        title: item.title,
        content: item.content,
        tags: item.tags,
        source: item.source,
        ..Default::default()
    }
}
