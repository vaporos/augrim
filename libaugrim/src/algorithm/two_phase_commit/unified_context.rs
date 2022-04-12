// Copyright 2021-2022 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::algorithm::two_phase_commit::Epoch;
use crate::error::InvalidStateError;
use crate::process::Process;
use crate::time::Time;

use super::TwoPhaseCommitState;
use super::{CoordinatorContext, CoordinatorState, Participant};
use super::{ParticipantContext, ParticipantState};

#[derive(Clone)]
enum InnerContext<P, T>
where
    P: Process,
    T: Time,
{
    Coordinator(CoordinatorContext<P, T>),
    Participant(ParticipantContext<P, T>),
}

#[derive(Clone)]
pub struct TwoPhaseCommitRoleContext<P, T>
where
    P: Process,
    T: Time,
{
    inner: InnerContext<P, T>,
}

impl<P, T> TryFrom<TwoPhaseCommitRoleContext<P, T>> for CoordinatorContext<P, T>
where
    P: Process,
    T: Time,
{
    type Error = InvalidStateError;

    fn try_from(context: TwoPhaseCommitRoleContext<P, T>) -> Result<Self, Self::Error> {
        match context.inner {
            InnerContext::Coordinator(c) => Ok(c),
            InnerContext::Participant(_) => Err(InvalidStateError::with_message(
                "unable to convert TwoPhaseCommitRoleContext to CoordinatorContext \
                because inner context type is Participant"
                    .into(),
            )),
        }
    }
}

impl<P, T> TryFrom<TwoPhaseCommitRoleContext<P, T>> for ParticipantContext<P, T>
where
    P: Process,
    T: Time,
{
    type Error = InvalidStateError;

    fn try_from(context: TwoPhaseCommitRoleContext<P, T>) -> Result<Self, Self::Error> {
        match context.inner {
            InnerContext::Participant(c) => Ok(c),
            InnerContext::Coordinator(_) => Err(InvalidStateError::with_message(
                "unable to convert TwoPhaseCommitRoleContext to ParticipantContext \
                because inner context type is Coordinator"
                    .into(),
            )),
        }
    }
}

impl<P, T> From<CoordinatorContext<P, T>> for TwoPhaseCommitRoleContext<P, T>
where
    P: Process,
    T: Time,
{
    fn from(context: CoordinatorContext<P, T>) -> Self {
        Self {
            inner: InnerContext::Coordinator(context),
        }
    }
}

impl<P, T> From<ParticipantContext<P, T>> for TwoPhaseCommitRoleContext<P, T>
where
    P: Process,
    T: Time,
{
    fn from(context: ParticipantContext<P, T>) -> Self {
        Self {
            inner: InnerContext::Participant(context),
        }
    }
}

#[derive(Default)]
pub struct TwoPhaseCommitContextBuilder<P, T>
where
    P: Process,
    T: Time,
{
    alarm: Option<T>,
    coordinator: Option<P>,
    epoch: Option<Epoch>,
    last_commit_epoch: Option<Epoch>,
    participants: Option<Vec<Participant<P>>>,
    participant_processes: Option<Vec<P>>,
    state: Option<TwoPhaseCommitState<T>>,
    this_process: Option<P>,
}

impl<P, T> TwoPhaseCommitContextBuilder<P, T>
where
    P: Process,
    T: Time,
{
    pub fn new() -> Self {
        Self {
            alarm: None,
            coordinator: None,
            epoch: None,
            last_commit_epoch: None,
            participants: None,
            participant_processes: None,
            state: None,
            this_process: None,
        }
    }

    pub fn with_alarm(mut self, alarm: T) -> Self {
        self.alarm = Some(alarm);
        self
    }

    pub fn with_coordinator(mut self, coordinator: P) -> Self {
        self.coordinator = Some(coordinator);
        self
    }

    pub fn with_epoch(mut self, epoch: Epoch) -> Self {
        self.epoch = Some(epoch);
        self
    }

    pub fn with_last_commit_epoch(mut self, last_commit_epoch: Epoch) -> Self {
        self.last_commit_epoch = Some(last_commit_epoch);
        self
    }

    pub fn with_participants(mut self, participants: Vec<Participant<P>>) -> Self {
        self.participants = Some(participants);
        self
    }

    pub fn with_participant_processes(mut self, participant_processes: Vec<P>) -> Self {
        self.participant_processes = Some(participant_processes);
        self
    }

    pub fn with_state(mut self, state: TwoPhaseCommitState<T>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_this_process(mut self, this_process: P) -> Self {
        self.this_process = Some(this_process);
        self
    }

    pub fn build(
        self,
    ) -> Result<TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>, InvalidStateError>
    {
        let alarm = self.alarm;
        let last_commit_epoch = self.last_commit_epoch;

        let coordinator = self
            .coordinator
            .ok_or_else(|| InvalidStateError::with_message("missing coordinator field".into()))?;

        let epoch = self
            .epoch
            .ok_or_else(|| InvalidStateError::with_message("missing epoch field".into()))?;

        let state = self
            .state
            .ok_or_else(|| InvalidStateError::with_message("missing state field".into()))?;

        let this_process = self
            .this_process
            .ok_or_else(|| InvalidStateError::with_message("missing this_process field".into()))?;

        let role_context = match (self.participants, self.participant_processes) {
            (Some(participants), None) => Ok(TwoPhaseCommitRoleContext {
                inner: InnerContext::Coordinator(CoordinatorContext {
                    participants,
                    state: state.try_into()?,
                }),
            }),
            (None, Some(participant_processes)) => Ok(TwoPhaseCommitRoleContext {
                inner: InnerContext::Participant(ParticipantContext {
                    participant_processes,
                    state: state.try_into()?,
                }),
            }),
            (Some(_), Some(_)) => Err(InvalidStateError::with_message(
                "participant and participant_processes fields are mutually exclusive".into(),
            )),
            (None, None) => Err(InvalidStateError::with_message(
                "exactly one of participant or particpant_processes fields required".into(),
            )),
        }?;

        Ok(TwoPhaseCommitContext {
            alarm,
            coordinator,
            epoch,
            last_commit_epoch,
            role_context,
            this_process,
        })
    }
}

#[derive(Clone)]
pub struct TwoPhaseCommitContext<P, T, R>
where
    P: Process,
    T: Time,
    R: Clone,
{
    alarm: Option<T>,
    coordinator: P,
    epoch: Epoch,
    last_commit_epoch: Option<Epoch>,
    pub(super) role_context: R,
    this_process: P,
}

impl<P, T, R> TwoPhaseCommitContext<P, T, R>
where
    P: Process,
    T: Time,
    R: Clone,
{
    pub fn alarm(&self) -> &Option<T> {
        &self.alarm
    }

    pub fn set_alarm(&mut self, alarm: Option<T>) {
        self.alarm = alarm;
    }

    pub fn coordinator(&self) -> &P {
        &self.coordinator
    }

    pub fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    pub fn set_epoch(&mut self, epoch: Epoch) {
        self.epoch = epoch
    }

    pub fn last_commit_epoch(&self) -> &Option<Epoch> {
        &self.last_commit_epoch
    }

    pub fn set_last_commit_epoch(&mut self, epoch: Option<Epoch>) {
        self.last_commit_epoch = epoch
    }

    pub fn this_process(&self) -> &P {
        &self.this_process
    }
}

impl<P, T> TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>
where
    P: Process,
    T: Time,
{
    pub fn participants(&self) -> Option<&Vec<Participant<P>>> {
        match &self.role_context.inner {
            InnerContext::Coordinator(c) => Some(&c.participants),
            InnerContext::Participant(_) => None,
        }
    }

    pub fn participant_processes(&self) -> Option<&Vec<P>> {
        match &self.role_context.inner {
            InnerContext::Coordinator(_) => None,
            InnerContext::Participant(c) => Some(&c.participant_processes),
        }
    }

    pub fn state(&self) -> TwoPhaseCommitState<T> {
        match &self.role_context.inner {
            InnerContext::Coordinator(c) => c.state.clone().into(),
            InnerContext::Participant(c) => c.state.clone().into(),
        }
    }
}

impl<P, T> TwoPhaseCommitContext<P, T, CoordinatorContext<P, T>>
where
    P: Process,
    T: Time,
{
    pub fn new(this_process: P, coordinator: P, participant_processes: Vec<P>) -> Self {
        Self {
            alarm: None,
            coordinator,
            epoch: 0,
            last_commit_epoch: None,
            role_context: CoordinatorContext {
                participants: participant_processes
                    .into_iter()
                    .map(Participant::new)
                    .collect(),
                state: CoordinatorState::WaitingForStart,
            },
            this_process,
        }
    }

    pub fn participants(&self) -> &Vec<Participant<P>> {
        &self.role_context.participants
    }

    pub fn participants_mut(&mut self) -> &mut Vec<Participant<P>> {
        &mut self.role_context.participants
    }

    pub fn state(&self) -> &CoordinatorState<T> {
        &self.role_context.state
    }

    pub fn set_state(&mut self, state: CoordinatorState<T>) {
        self.role_context.state = state;
    }
}

impl<P, T> TwoPhaseCommitContext<P, T, ParticipantContext<P, T>>
where
    P: Process,
    T: Time,
{
    pub fn new(this_process: P, coordinator: P, participant_processes: Vec<P>) -> Self {
        Self {
            alarm: None,
            coordinator,
            epoch: 0,
            last_commit_epoch: None,
            role_context: ParticipantContext {
                participant_processes,
                state: ParticipantState::WaitingForVoteRequest,
            },
            this_process,
        }
    }

    pub fn participant_processes(&self) -> &Vec<P> {
        &self.role_context.participant_processes
    }

    pub fn state(&self) -> &ParticipantState<T> {
        &self.role_context.state
    }

    pub fn set_state(&mut self, state: ParticipantState<T>) {
        self.role_context.state = state;
    }
}

impl<P, T> TryFrom<TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>>
    for TwoPhaseCommitContext<P, T, CoordinatorContext<P, T>>
where
    P: Process,
    T: Time,
{
    type Error = InvalidStateError;

    fn try_from(
        context: TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            alarm: context.alarm,
            coordinator: context.coordinator,
            epoch: context.epoch,
            last_commit_epoch: context.last_commit_epoch,
            role_context: context.role_context.try_into()?,
            this_process: context.this_process,
        })
    }
}

impl<P, T> TryFrom<TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>>
    for TwoPhaseCommitContext<P, T, ParticipantContext<P, T>>
where
    P: Process,
    T: Time,
{
    type Error = InvalidStateError;

    fn try_from(
        context: TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            alarm: context.alarm,
            coordinator: context.coordinator,
            epoch: context.epoch,
            last_commit_epoch: context.last_commit_epoch,
            role_context: context.role_context.try_into()?,
            this_process: context.this_process,
        })
    }
}

impl<P, T> From<TwoPhaseCommitContext<P, T, CoordinatorContext<P, T>>>
    for TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>
where
    P: Process,
    T: Time,
{
    fn from(context: TwoPhaseCommitContext<P, T, CoordinatorContext<P, T>>) -> Self {
        Self {
            alarm: context.alarm,
            coordinator: context.coordinator,
            epoch: context.epoch,
            last_commit_epoch: context.last_commit_epoch,
            role_context: context.role_context.into(),
            this_process: context.this_process,
        }
    }
}

impl<P, T> From<TwoPhaseCommitContext<P, T, ParticipantContext<P, T>>>
    for TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>
where
    P: Process,
    T: Time,
{
    fn from(context: TwoPhaseCommitContext<P, T, ParticipantContext<P, T>>) -> Self {
        Self {
            alarm: context.alarm,
            coordinator: context.coordinator,
            epoch: context.epoch,
            last_commit_epoch: context.last_commit_epoch,
            role_context: context.role_context.into(),
            this_process: context.this_process,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;

    impl Process for String {}

    #[test]
    fn build_coordinator_context() {
        let now = SystemTime::now();

        let unified_context = TwoPhaseCommitContextBuilder::<String, SystemTime>::new()
            .with_alarm(now)
            .with_coordinator("me".into())
            .with_epoch(2)
            .with_last_commit_epoch(1)
            .with_state(TwoPhaseCommitState::WaitingForStart)
            .with_this_process("me".into())
            .with_participants(vec![
                Participant::new("me".into()),
                Participant::new("p1".into()),
                Participant::new("p2".into()),
            ])
            .build()
            .unwrap();

        let coordinator_context: TwoPhaseCommitContext<_, _, CoordinatorContext<_, _>> =
            unified_context.try_into().unwrap();

        assert_eq!(coordinator_context.alarm().unwrap(), now);
        assert_eq!(*coordinator_context.coordinator(), "me".to_string());
        assert_eq!(*coordinator_context.epoch(), 2);
        assert_eq!(coordinator_context.last_commit_epoch().unwrap(), 1);
        assert_eq!(
            *coordinator_context.state(),
            CoordinatorState::WaitingForStart
        );
        assert_eq!(coordinator_context.participants().len(), 3);

        let reunified_context: TwoPhaseCommitContext<_, _, TwoPhaseCommitRoleContext<_, _>> =
            coordinator_context.into();

        assert_eq!(reunified_context.alarm().unwrap(), now);
        assert_eq!(*reunified_context.coordinator(), "me".to_string());
        assert_eq!(*reunified_context.epoch(), 2);
        assert_eq!(reunified_context.last_commit_epoch().unwrap(), 1);
        assert_eq!(
            reunified_context.state(),
            TwoPhaseCommitState::WaitingForStart
        );
        assert_eq!(reunified_context.participants().unwrap().len(), 3);
    }
}
