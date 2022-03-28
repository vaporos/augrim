// Copyright 2021 Cargill Incorporated
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

use std::marker::PhantomData;

use crate::algorithm::{Algorithm, Value};
use crate::error::InternalError;
use crate::process::Process;

use super::action::FloodingAction;
use super::context::FloodingContext;
use super::event::FloodingEvent;
use super::message::FloodingMessage;
use super::Round;

pub struct FloodingAlgorithm<P, V, F>
where
    P: Process,
    V: Value,
    F: Fn(&Vec<V>) -> Result<V, InternalError>,
{
    select_func: F,
    _process_phantom: PhantomData<P>,
    _value_phantom: PhantomData<V>,
}

impl<P, V, F> FloodingAlgorithm<P, V, F>
where
    P: Process,
    V: Value,
    F: Fn(&Vec<V>) -> Result<V, InternalError>,
{
    pub fn new(select_func: F) -> Self {
        FloodingAlgorithm {
            select_func,
            _process_phantom: PhantomData,
            _value_phantom: PhantomData,
        }
    }

    // When a crash event is received, remove the crashed process from the list of correct
    // processes and then check if this change causes us to advance.
    fn handle_crash(
        &self,
        mut context: FloodingContext<P, V>,
        process: P,
    ) -> Result<Vec<FloodingAction<P, V>>, InternalError> {
        Ok(match context.correct().iter().position(|p| *p == process) {
            None => Vec::new(),
            Some(i) => {
                context.correct_mut().swap_remove(i);

                let mut actions = Vec::new();
                self.decide_or_next_round(&mut context, &mut actions)?;
                actions.insert(0, FloodingAction::UpdateContext(context));
                actions
            }
        })
    }

    // A propose event occurs when the local process proposes a value to consensus (as opposed to
    // receiving proposals broadcast by remote processes). The proposed value is added to the list
    // of proposals for round 1, then we broadcast our list of round 1 proposals to the other
    // nodes.
    fn handle_propose(
        &self,
        mut context: FloodingContext<P, V>,
        value: V,
    ) -> Result<Vec<FloodingAction<P, V>>, InternalError> {
        context.proposals_mut()[1].push(value);

        let mut actions = vec![FloodingAction::Broadcast(FloodingMessage::Proposal(
            1,
            context.proposals()[1].clone(),
        ))];
        actions.insert(0, FloodingAction::UpdateContext(context));
        Ok(actions)
    }

    // Having received a proposal message from a remote process:
    //   - add the process to context.received_from[context.round]
    //   - add the proposed values to context.proposals[current.round]
    //
    // If round from the remote process is the same as context.round (our current round), then
    // context.received_from[context.round] has changed, so we call decide_or_next_round().
    fn handle_deliver_proposal(
        &self,
        mut context: FloodingContext<P, V>,
        process: P,
        round: Round,
        mut proposals: Vec<V>,
    ) -> Result<Vec<FloodingAction<P, V>>, InternalError> {
        context.received_from_mut()[round].push(process);
        context.proposals_mut()[round].append(&mut proposals);

        let mut actions = Vec::new();
        if round == context.round() {
            self.decide_or_next_round(&mut context, &mut actions)?;
        }
        actions.insert(0, FloodingAction::UpdateContext(context));
        Ok(actions)
    }

    // If conditions are met, make a decision or advance to the next round.
    //
    // This method should be called after any of the following are changed within the context:
    //   - context.correct
    //   - context.received_from
    //   - context.decision
    //
    // This call is a no-op unless the following conditions are all true:
    //   - context.correct is a subset of context.received_from[context.round]
    //   - context.decision is None
    //
    // If the above conditions are met and context.received_from[round] is the same as
    // received_from[round-1], since that condition will be true if context.correct has not changed
    // this round. If context.correct has changed, then we move to the next round.
    fn decide_or_next_round(
        &self,
        context: &mut FloodingContext<P, V>,
        actions: &mut Vec<FloodingAction<P, V>>,
    ) -> Result<(), InternalError> {
        // If context.correct is a subset of context.received_from[context.round] and
        // context.decision is None, then we can either decide or increment the round.
        if !context
            .correct()
            .iter()
            .any(|p| !context.received_from()[context.round()].contains(p))
            && context.decision().is_none()
        {
            // If context.received[context.round] has the same processes as in the previous round,
            // we can decide; otherwise we will increment the round.
            if context.received_from()[context.round()].len()
                == context.received_from()[context.round() - 1].len()
                && !context.received_from()[context.round()]
                    .iter()
                    .any(|p| !context.received_from()[context.round() - 1].contains(p))
            {
                let decision = (self.select_func)(&context.proposals()[context.round()])?;
                context.set_decision(Some(decision.clone()));
                actions.push(FloodingAction::Broadcast(FloodingMessage::Decided(
                    decision.clone(),
                )));
                actions.push(FloodingAction::Decide(decision));
            } else {
                context.set_round(context.round() + 1);

                // Broadcast [Proposal, round, proposals[round - 1]]
                actions.push(FloodingAction::Broadcast(FloodingMessage::Proposal(
                    context.round(),
                    context.proposals()[context.round() - 1].clone(),
                )));
            }
        }
        Ok(())
    }

    // Process a decided message from a remote process.
    //
    // This method is a no-op unless the following conditions are all true:
    //   - the remote process is in context.correct
    //   - context.decision is None
    //
    // If the above conditions are met, we accept the decision and.
    fn handle_deliver_decided(
        &self,
        mut context: FloodingContext<P, V>,
        process: P,
        decision: V,
    ) -> Result<Vec<FloodingAction<P, V>>, InternalError> {
        let mut actions = Vec::new();

        if context.correct().contains(&process) && context.decision().is_none() {
            context.set_decision(Some(decision.clone()));
            actions.push(FloodingAction::Broadcast(FloodingMessage::Decided(
                decision.clone(),
            )));
            actions.push(FloodingAction::Decide(decision));
        }

        Ok(actions)
    }
}

impl<P, V, F> Algorithm<P> for FloodingAlgorithm<P, V, F>
where
    P: Process,
    V: Value,
    F: Fn(&Vec<V>) -> Result<V, InternalError>,
{
    type Event = FloodingEvent<P, V>;
    type Action = FloodingAction<P, V>;
    type Context = FloodingContext<P, V>;

    fn event(
        &self,
        event: Self::Event,
        context: Self::Context,
    ) -> Result<Vec<Self::Action>, InternalError> {
        match event {
            FloodingEvent::Crash(p) => self.handle_crash(context, p),
            FloodingEvent::Deliver(p, m) => match m {
                FloodingMessage::Proposal(round, values) => {
                    self.handle_deliver_proposal(context, p, round, values)
                }
                FloodingMessage::Decided(value) => self.handle_deliver_decided(context, p, value),
            },
            FloodingEvent::Propose(v) => self.handle_propose(context, v),
        }
    }
}
