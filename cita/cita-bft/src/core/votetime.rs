// CITA
// Copyright 2016-2017 Cryptape Technologies LLC.

// This program is free software: you can redistribute it
// and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option) any
// later version.

// This program is distributed in the hope that it will be
// useful, but WITHOUT ANY WARRANTY; without even the implied
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
// PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::core::cita_bft::{BftTurn, Step};
use min_max_heap::MinMaxHeap;
use pubsub::channel::{Receiver, Sender};
use std::cmp::Ordering;
use std::time::{Duration, Instant};

#[derive(Clone, Eq, PartialEq)]
pub struct TimeoutInfo {
    pub timeval: Instant,
    pub height: usize,
    pub round: usize,
    pub step: Step,
}

impl ::std::cmp::PartialOrd for TimeoutInfo {
    fn partial_cmp(&self, other: &TimeoutInfo) -> Option<Ordering> {
        self.timeval.partial_cmp(&other.timeval)
    }
}

impl ::std::cmp::Ord for TimeoutInfo {
    fn cmp(&self, other: &TimeoutInfo) -> Ordering {
        self.timeval.cmp(&other.timeval)
    }
}

impl ::std::fmt::Display for TimeoutInfo {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(
            f,
            "TimeoutInfo {{ h: {}, r: {}, s: {}, t: {:?} }}",
            self.height,
            self.round,
            self.step,
            self.timeval.elapsed()
        )
    }
}

impl ::std::fmt::Debug for TimeoutInfo {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub struct WaitTimer {
    timer_seter: Receiver<TimeoutInfo>,
    timer_notify: Sender<BftTurn>,
}

impl WaitTimer {
    pub fn new(ts: Sender<BftTurn>, rs: Receiver<TimeoutInfo>) -> WaitTimer {
        WaitTimer {
            timer_notify: ts,
            timer_seter: rs,
        }
    }

    pub fn start(&self) {
        let mut timer_heap = MinMaxHeap::<TimeoutInfo>::new();

        loop {
            // take the peek of the min-heap-timer sub now as the sleep time otherwise set timeout as 100
            let timeout = if !timer_heap.is_empty() {
                let peek_min_interval = timer_heap.peek_min().unwrap().timeval;
                let now = Instant::now();
                if peek_min_interval > now {
                    peek_min_interval - now
                } else {
                    Duration::new(0, 0)
                }
            } else {
                Duration::from_secs(100)
            };

            let set_time = self.timer_seter.recv_timeout(timeout);

            // put the timeval into a timerheap
            // put the TimeoutInfo into a hashmap, K: timeval  V: TimeoutInfo
            if set_time.is_ok() {
                let time_out = set_time.unwrap();
                timer_heap.push(time_out);
            }

            if !timer_heap.is_empty() {
                let now = Instant::now();

                // if some timers are set as the same time, send timeout messages and pop them
                while !timer_heap.is_empty()
                    && now >= timer_heap.peek_min().cloned().unwrap().timeval
                {
                    self.timer_notify
                        .send(BftTurn::Timeout(timer_heap.pop_min().unwrap()))
                        .unwrap();
                }
            }
        }
    }
}
