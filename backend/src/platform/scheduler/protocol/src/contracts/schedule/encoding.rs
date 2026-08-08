use makosh_clock_protocol::{TimeZoneContextV1, UtcMillisV1};

use super::{
    MisfirePolicyV1, OverlapPolicyV1, RetryPolicyV1, ScheduleCodecErrorV1, SchedulePolicyV1,
    ScheduleTriggerV1,
};

const VERSION: u8 = 1;
const TRIGGER_AT: u8 = 1;
const TRIGGER_INTERVAL: u8 = 2;
const TRIGGER_DELAY: u8 = 3;
const TRIGGER_CRON: u8 = 4;
const OVERLAP_FORBID: u8 = 1;
const OVERLAP_QUEUE: u8 = 2;
const OVERLAP_COALESCE: u8 = 3;
const OVERLAP_BOUNDED: u8 = 4;
const MISFIRE_SKIP: u8 = 1;
const MISFIRE_ONCE: u8 = 2;
const MISFIRE_BOUNDED: u8 = 3;

pub(super) fn encode(policy: &SchedulePolicyV1) -> Vec<u8> {
    let mut output = Vec::with_capacity(64);
    output.push(VERSION);
    encode_trigger(&mut output, policy.trigger());
    encode_overlap(&mut output, policy.overlap());
    encode_misfire(&mut output, policy.misfire());
    encode_retry(&mut output, policy.retry());
    output.extend_from_slice(&policy.deadline_millis().to_be_bytes());
    output.extend_from_slice(&policy.jitter_millis().to_be_bytes());
    output
}

pub(super) fn decode(bytes: &[u8]) -> Result<SchedulePolicyV1, ScheduleCodecErrorV1> {
    let mut input = DecoderV1::new(bytes);
    if input.u8()? != VERSION {
        return Err(ScheduleCodecErrorV1::InvalidEncoding);
    }
    let trigger = decode_trigger(&mut input)?;
    let overlap = decode_overlap(&mut input)?;
    let misfire = decode_misfire(&mut input)?;
    let retry = decode_retry(&mut input)?;
    let deadline = input.u64()?;
    let jitter = input.u64()?;
    if !input.finished() {
        return Err(ScheduleCodecErrorV1::InvalidEncoding);
    }
    SchedulePolicyV1::new(trigger, overlap, misfire, retry, deadline, jitter)
        .map_err(|_| ScheduleCodecErrorV1::InvalidEncoding)
}

fn encode_trigger(output: &mut Vec<u8>, trigger: &ScheduleTriggerV1) {
    match trigger {
        ScheduleTriggerV1::At { due_at } => {
            output.push(TRIGGER_AT);
            output.extend_from_slice(&due_at.value().to_be_bytes());
        }
        ScheduleTriggerV1::FixedInterval { interval_millis } => {
            output.push(TRIGGER_INTERVAL);
            output.extend_from_slice(&interval_millis.to_be_bytes());
        }
        ScheduleTriggerV1::FixedDelay { delay_millis } => {
            output.push(TRIGGER_DELAY);
            output.extend_from_slice(&delay_millis.to_be_bytes());
        }
        ScheduleTriggerV1::Cron {
            expression,
            timezone,
        } => encode_cron(output, expression, timezone),
    }
}

fn encode_cron(output: &mut Vec<u8>, expression: &str, timezone: &TimeZoneContextV1) {
    output.push(TRIGGER_CRON);
    encode_string(output, expression);
    encode_string(output, timezone.iana_name());
    output.extend_from_slice(&timezone.utc_offset_seconds().to_be_bytes());
}

fn encode_overlap(output: &mut Vec<u8>, overlap: OverlapPolicyV1) {
    match overlap {
        OverlapPolicyV1::Forbid => output.push(OVERLAP_FORBID),
        OverlapPolicyV1::Queue { max_pending_runs } => {
            output.push(OVERLAP_QUEUE);
            output.extend_from_slice(&max_pending_runs.to_be_bytes());
        }
        OverlapPolicyV1::CoalesceLatest => output.push(OVERLAP_COALESCE),
        OverlapPolicyV1::AllowBounded { max_parallelism } => {
            output.push(OVERLAP_BOUNDED);
            output.extend_from_slice(&max_parallelism.to_be_bytes());
        }
    }
}

fn encode_misfire(output: &mut Vec<u8>, misfire: MisfirePolicyV1) {
    match misfire {
        MisfirePolicyV1::Skip => output.push(MISFIRE_SKIP),
        MisfirePolicyV1::FireOnce => output.push(MISFIRE_ONCE),
        MisfirePolicyV1::CatchUpBounded { max_runs } => {
            output.push(MISFIRE_BOUNDED);
            output.extend_from_slice(&max_runs.to_be_bytes());
        }
    }
}

fn encode_retry(output: &mut Vec<u8>, retry: RetryPolicyV1) {
    output.extend_from_slice(&retry.max_attempts().to_be_bytes());
    output.extend_from_slice(&retry.base_backoff_millis().to_be_bytes());
}

fn encode_string(output: &mut Vec<u8>, value: &str) {
    let length = u16::try_from(value.len()).expect("Scheduler string is bounded by its contract");
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value.as_bytes());
}

fn decode_trigger(input: &mut DecoderV1<'_>) -> Result<ScheduleTriggerV1, ScheduleCodecErrorV1> {
    match input.u8()? {
        TRIGGER_AT => Ok(ScheduleTriggerV1::At {
            due_at: UtcMillisV1::new(input.i64()?),
        }),
        TRIGGER_INTERVAL => Ok(ScheduleTriggerV1::FixedInterval {
            interval_millis: input.u64()?,
        }),
        TRIGGER_DELAY => Ok(ScheduleTriggerV1::FixedDelay {
            delay_millis: input.u64()?,
        }),
        TRIGGER_CRON => decode_cron(input),
        _ => Err(ScheduleCodecErrorV1::InvalidEncoding),
    }
}

fn decode_cron(input: &mut DecoderV1<'_>) -> Result<ScheduleTriggerV1, ScheduleCodecErrorV1> {
    let expression = input.string()?;
    let timezone_name = input.string()?;
    let timezone = TimeZoneContextV1::new(timezone_name, input.i32()?)
        .map_err(|_| ScheduleCodecErrorV1::InvalidEncoding)?;
    Ok(ScheduleTriggerV1::Cron {
        expression,
        timezone,
    })
}

fn decode_overlap(input: &mut DecoderV1<'_>) -> Result<OverlapPolicyV1, ScheduleCodecErrorV1> {
    match input.u8()? {
        OVERLAP_FORBID => Ok(OverlapPolicyV1::Forbid),
        OVERLAP_QUEUE => Ok(OverlapPolicyV1::Queue {
            max_pending_runs: input.u16()?,
        }),
        OVERLAP_COALESCE => Ok(OverlapPolicyV1::CoalesceLatest),
        OVERLAP_BOUNDED => Ok(OverlapPolicyV1::AllowBounded {
            max_parallelism: input.u16()?,
        }),
        _ => Err(ScheduleCodecErrorV1::InvalidEncoding),
    }
}

fn decode_misfire(input: &mut DecoderV1<'_>) -> Result<MisfirePolicyV1, ScheduleCodecErrorV1> {
    match input.u8()? {
        MISFIRE_SKIP => Ok(MisfirePolicyV1::Skip),
        MISFIRE_ONCE => Ok(MisfirePolicyV1::FireOnce),
        MISFIRE_BOUNDED => Ok(MisfirePolicyV1::CatchUpBounded {
            max_runs: input.u16()?,
        }),
        _ => Err(ScheduleCodecErrorV1::InvalidEncoding),
    }
}

fn decode_retry(input: &mut DecoderV1<'_>) -> Result<RetryPolicyV1, ScheduleCodecErrorV1> {
    RetryPolicyV1::new(input.u16()?, input.u64()?)
        .map_err(|_| ScheduleCodecErrorV1::InvalidEncoding)
}

struct DecoderV1<'a> {
    remaining: &'a [u8],
}

impl<'a> DecoderV1<'a> {
    const fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }

    const fn finished(&self) -> bool {
        self.remaining.is_empty()
    }

    fn u8(&mut self) -> Result<u8, ScheduleCodecErrorV1> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, ScheduleCodecErrorV1> {
        Ok(u16::from_be_bytes(
            self.take(2)?.try_into().expect("exact length"),
        ))
    }

    fn u64(&mut self) -> Result<u64, ScheduleCodecErrorV1> {
        Ok(u64::from_be_bytes(
            self.take(8)?.try_into().expect("exact length"),
        ))
    }

    fn i64(&mut self) -> Result<i64, ScheduleCodecErrorV1> {
        Ok(i64::from_be_bytes(
            self.take(8)?.try_into().expect("exact length"),
        ))
    }

    fn i32(&mut self) -> Result<i32, ScheduleCodecErrorV1> {
        Ok(i32::from_be_bytes(
            self.take(4)?.try_into().expect("exact length"),
        ))
    }

    fn string(&mut self) -> Result<String, ScheduleCodecErrorV1> {
        let length = usize::from(self.u16()?);
        let bytes = self.take(length)?;
        let value =
            std::str::from_utf8(bytes).map_err(|_| ScheduleCodecErrorV1::InvalidEncoding)?;
        Ok(value.to_owned())
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], ScheduleCodecErrorV1> {
        let Some((value, remaining)) = self.remaining.split_at_checked(length) else {
            return Err(ScheduleCodecErrorV1::InvalidEncoding);
        };
        self.remaining = remaining;
        Ok(value)
    }
}
