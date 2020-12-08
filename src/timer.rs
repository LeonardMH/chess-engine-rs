type PlayerCount = usize;
type PlayerIndex = usize;

pub const SUPPORTED_PLAYERS: PlayerCount = 2;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TimerDirection {
    Down,
    Up,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TimerError {
    SettingsConflict(String),
}

struct ChessTimer<'a> {
    started_at: Option<std::time::Instant>,
    last_player_switch_at: Option<std::time::Instant>,
    direction: TimerDirection,

    curr_player_index: Option<PlayerIndex>,
    last_player_index: Option<PlayerIndex>,

    player_elapsed_ms: [i64; SUPPORTED_PLAYERS],
    player_maxtime_ms: [u32; SUPPORTED_PLAYERS],
    player_adjust_on_switch_ms: [i64; SUPPORTED_PLAYERS],

    callback: Box<dyn FnMut(PlayerIndex) + 'a>,
}

type Result<T> = std::result::Result<T, TimerError>;

impl<'a> ChessTimer<'a> {
    pub fn new(direction: TimerDirection,
           player_maxtime_ms: Option<[u32; SUPPORTED_PLAYERS]>,
           player_adjust_on_switch_ms: Option<[i64; SUPPORTED_PLAYERS]>) -> Result<ChessTimer<'a>>{

        let player_maxtime_ms = match player_maxtime_ms {
            Some(maxtime) => maxtime,
            None => {
                if direction == TimerDirection::Down {
                    let string = "Down counting timer requires a maxtime".to_string();
                    return Err(TimerError::SettingsConflict(string));
                }

                [0; SUPPORTED_PLAYERS]
            },
        };

        let player_adjust_on_switch_ms = match player_adjust_on_switch_ms {
            Some(adjust) => adjust,
            None => [0; SUPPORTED_PLAYERS],
        };

        Ok(ChessTimer{
            started_at: None,
            last_player_switch_at: None,
            direction,

            curr_player_index: Some(0),
            last_player_index: None,

            player_elapsed_ms: [0; SUPPORTED_PLAYERS],
            player_maxtime_ms,
            player_adjust_on_switch_ms,
            callback: Box::new(|_: PlayerIndex| ()),
        })
    }

    pub fn set_callback(&mut self, c: impl FnMut(PlayerIndex) + 'a) {
        self.callback = Box::new(c);
    }

    fn trigger_callback(&mut self, player: PlayerIndex) {
        (self.callback)(player);
    }

    pub fn start(&mut self) {
        // capture the time at the start of the function for consistency
        let now = std::time::Instant::now();

        // if the timer has already started then early return so as to not reset it
        if self.started_at.is_some() {
            return;
        }

        // if we previously stopped the timer then the last_player_index will have a
        // value, we want to restart the timer with that player active
        if let Some(last_player) = self.last_player_index {
            self.curr_player_index = Some(last_player);
            self.last_player_index = None;
        }

        self.started_at = Some(now);
        self.last_player_switch_at = Some(now);
    }

    pub fn stop(&mut self) {
        // capture the time at the start of the function for consistency
        let now = std::time::Instant::now();

        // only stop the counter if it is already running, otherwise the code becomes a pain
        if self.started_at.is_none() {
            return;
        }

        // commit the statistics of the current player (assuming there is one)
        if let Some(current_player) = self.curr_player_index {
            // safe to unwrap self.started_at as we have already verified it to be Some
            let benchmark = self.last_player_switch_at.unwrap_or_else(|| {
                self.started_at.unwrap()
            });

            self.adjust_elapsed_time_for_player(current_player, benchmark.elapsed().as_millis() as i64);
            self.last_player_switch_at = Some(now);

            // invalidate indicators of timer progression
            self.started_at = None;
        }
    }

    fn player_index_supported(player: PlayerIndex) -> bool {
        player < SUPPORTED_PLAYERS
    }

    fn elapsed_to_remaining(elapsed: i64, last_remaining: u32) -> u32 {
        // if elapsed time is larger than (or equal to) last_remaining then simply return 0, indicating
        // that the player has no remaining time
        //
        // safe to upcast a u32 to an i64
        if elapsed >= last_remaining as i64 {
            return 0;
        }

        // if elapsed time is very deeply negative (indicating that we are adding time back
        // to the player's counter) then it could potentially cause the last_remaining to overflow
        // we can check for this by finding the maximum allowable value based on what is
        // already in last_remaining
        if elapsed.is_negative() {
            let max_allowed_timelapse = u32::MAX - last_remaining;
            if elapsed.abs() >= max_allowed_timelapse as i64 {
                return u32::MAX;
            }

            elapsed.abs() as u32 + last_remaining
        } else {
            // now that we are sure elapsed is numerically smaller than last_remaining and that the
            // overall result will fit in a u32 we can safely downcast `elapsed` to a u32
            last_remaining - elapsed as u32
        }
    }

    pub fn check_elapsed_time_for_player(&self, player: PlayerIndex) -> Option<i64> {
        if !Self::player_index_supported(player) {
            return None;
        }

        Some(self.player_elapsed_ms[player])
    }

    pub fn check_remaining_time_for_player(&self, player: PlayerIndex) -> Option<u32> {
        // this function call checks that player index is valid, so we don't have to do it
        // elsewhere in this function
        if let Some(elapsed) = self.check_elapsed_time_for_player(player) {
            Some(Self::elapsed_to_remaining(elapsed, self.player_maxtime_ms[player]))
        } else {
            None
        }
    }

    pub fn adjust_elapsed_time_for_player(&mut self, player: PlayerIndex, adjustment_ms: i64) {
        // do not panic if player index is out of bounds, simply do nothing
        if !Self::player_index_supported(player) {
            return;
        }

        // adjust player time, then handle side effects,
        if self.direction == TimerDirection::Down {
            // elapsed time is not allowed to be larger than maxtime for Down count timers
            self.player_elapsed_ms[player] = std::cmp::min(
                self.player_maxtime_ms[player].into(),
                self.player_elapsed_ms[player] + adjustment_ms);
        } else {
            self.player_elapsed_ms[player] += adjustment_ms;
        }

        // if the time adjustment makes the elapsed time meet or exceed the maxtime then
        // this player's time has expired
        //
        // `as i64` is safe in this case as we are upcasting from a u32
        if self.player_elapsed_ms[player] >= self.player_maxtime_ms[player].into() {
            self.trigger_callback(player);
            self.stop();
        }
    }

    pub fn current_player(&self) -> Option<PlayerIndex> { self.curr_player_index }

    pub fn switch_to_player(&mut self, player: PlayerIndex) {
        // capture the time at the start of the function for consistency
        let now = std::time::Instant::now();

        // this function does not raise an error on switching to an invalid
        // player index, it just does nothing
        if !Self::player_index_supported(player) {
            return;
        }

        // update the statistics of the player we are switching from
        if let Some(last_player_switch_at) = self.last_player_switch_at {
            if let Some(current_player) = self.curr_player_index {
                let last_switch = last_player_switch_at.elapsed().as_millis() as i64;
                let adjust_on_switch = self.player_adjust_on_switch_ms[current_player];

                self.adjust_elapsed_time_for_player(current_player, last_switch - adjust_on_switch);
            }
        }

        // now switch active players
        self.last_player_index = self.curr_player_index;
        self.last_player_switch_at = Some(now);
        self.curr_player_index = Some(player);
    }

    pub fn switch_to_next_player(&mut self) {
        // first check that we have a current player, if not this function does nothing
        if let Some(current_player) = self.curr_player_index {
            let provisional = current_player + 1;
            let next = if provisional >= SUPPORTED_PLAYERS { 0 } else { provisional };

            self.switch_to_player(next);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::timer::{ChessTimer, TimerDirection, SUPPORTED_PLAYERS};
    use std::time::Duration;

    #[test]
    fn test_timer_start_stop_restart() {
        let timer_result = ChessTimer::new(
            TimerDirection::Down,
            Some([1 * 1000; SUPPORTED_PLAYERS]),
            None);

        // verify that the timer construction was valid
        assert!(timer_result.is_ok());
        let mut timer = timer_result.unwrap(); // now safe to unwrap as we have tested for validity

        // start the timer and check that player 0 is the active player
        timer.start();
        assert_eq!(timer.current_player(), Some(0));
        std::thread::sleep(Duration::from_millis(10));

        // stop the timer and get the time for the current player
        timer.stop();
        assert_eq!(timer.current_player(), Some(0));
        let elapsed = timer.check_elapsed_time_for_player(timer.current_player().unwrap());

        // restart, wait, and stop the timer and check that more time has elapsed since the last
        // time we stopped the timer
        timer.start();
        std::thread::sleep(Duration::from_millis(10));
        timer.stop();
        assert_eq!(timer.current_player(), Some(0));
        let elapsed_after_restart = timer.check_elapsed_time_for_player(timer.current_player().unwrap());
        assert_gt!(elapsed_after_restart, elapsed);
    }

    #[test]
    fn test_elapsed_to_remaining() {
        // a basic and simple sanity check
        let result = ChessTimer::elapsed_to_remaining(10, 1000);
        assert_eq!(result, 990);

        // check basic negative elapse (when time is gained)
        let result = ChessTimer::elapsed_to_remaining(-4000, 1000);
        assert_eq!(result, 5000);

        // check very large values of elapsed which should result in bottoming out
        let result = ChessTimer::elapsed_to_remaining(u32::MAX as i64 + 10, 1000);
        assert_eq!(result, 0);

        // check case where a deeply negative elapsed time would normally overflow calculation
        let result = ChessTimer::elapsed_to_remaining(-(u32::MAX as i64) - 100, 1000);
        assert_eq!(result, u32::MAX);
    }

    #[test]
    fn test_player_cycle() {
        let test_maxtime_ms = 1000;
        let timer_result = ChessTimer::new(
            TimerDirection::Down,
            Some([test_maxtime_ms; SUPPORTED_PLAYERS]),
            None);

        // verify that the timer construction was valid
        assert!(timer_result.is_ok());
        let mut timer = timer_result.unwrap(); // now safe to unwrap as we have tested for validity

        // start the timer then cycle through each player (give a few milliseconds to allow the
        // counter to actually change)
        timer.start();
        std::thread::sleep(std::time::Duration::from_millis(50));

        const INTER_PLAYER_DELAY: u64 = 5;

        // loop through each player, committing a bit of time for each
        for index in 0..SUPPORTED_PLAYERS {
            timer.switch_to_player(index);
            std::thread::sleep(std::time::Duration::from_millis(INTER_PLAYER_DELAY));
        }

        // do it again with the automatic player switching function
        for _ in 0..SUPPORTED_PLAYERS {
            timer.switch_to_next_player();
            std::thread::sleep(std::time::Duration::from_millis(INTER_PLAYER_DELAY));
        }

        // stop the timer then cycle through players again and ensure that each one has some
        // amount of elapsed time
        timer.stop();

        let mut elapsed_at_stop = [0 as i64; SUPPORTED_PLAYERS];
        let mut remain_at_stop = [0 as u32; SUPPORTED_PLAYERS];

        for index in 0..SUPPORTED_PLAYERS {
            elapsed_at_stop[index] = timer.check_elapsed_time_for_player(index).unwrap();
            remain_at_stop [index] = timer.check_remaining_time_for_player(index).unwrap();

            // this test doesn't actually test the accuracy of the clock, mainly because I don't
            // know how to do that level of reliably introspection in my OS. I need an accurate
            // timer to compare to. Furthermore, the test itself uses sleep() to introduce a wait,
            // and sleep is not particularly precise
            assert!(elapsed_at_stop[index] > 0);
            assert_ne!(remain_at_stop[index], test_maxtime_ms);
        }

        // wait a little bit after stopping the timer so we can check whether it has truly
        // stopped tracking time
        std::thread::sleep(std::time::Duration::from_millis(4 * INTER_PLAYER_DELAY));

        // check that elapsed time is non-zero and that remaining time is non-maxtime
        for index in 0..SUPPORTED_PLAYERS {
            let elapsed = timer.check_elapsed_time_for_player(index).unwrap();
            let remain = timer.check_remaining_time_for_player(index).unwrap();

            assert_eq!(elapsed, elapsed_at_stop[index]);
            assert_eq!(remain, remain_at_stop[index]);
        }
    }

    #[test]
    fn test_manual_time_addition() {
        let mut timer = ChessTimer::new(
            TimerDirection::Down,
            Some([1000; SUPPORTED_PLAYERS]),
            None).unwrap();

        // no need to ever start the timer, just adjust player 0 elapsed time and check that
        // it is reported correctly
        timer.adjust_elapsed_time_for_player(0, 100);
        assert_eq!(timer.check_elapsed_time_for_player(0).unwrap(), 100);
    }

    #[test]
    fn test_manual_time_subtraction() {
        let mut timer = ChessTimer::new(
            TimerDirection::Down,
            Some([1000; SUPPORTED_PLAYERS]),
            None).unwrap();

        // no need to ever start the timer, just adjust player 0 elapsed time and check that
        // it is reported correctly
        timer.adjust_elapsed_time_for_player(0, -100);
        assert_eq!(timer.check_elapsed_time_for_player(0).unwrap(), -100);
    }

    #[test]
    fn test_time_addition_on_switch() {
        let test_maxtime_ms = 1000;
        let mut timer= ChessTimer::new(
            TimerDirection::Down,
            Some([test_maxtime_ms; SUPPORTED_PLAYERS]),
            Some([5 * 1000; SUPPORTED_PLAYERS])).unwrap();

        // start the timer and check that player 0 is the active player
        timer.start();
        assert_eq!(timer.current_player(), Some(0));

        // wait a bit for the timer to proceed
        std::thread::sleep(Duration::from_millis(40));

        // switch players and stop the timer, get the time for the previous player and ensure there
        // is more remaining time that what we started with
        timer.switch_to_player(1);
        timer.stop();

        assert_eq!(timer.current_player(), Some(1));

        // first check that elapsed time has moved in the correct direction, it should be
        // negative since we gained more time than used
        let elapsed = timer.check_elapsed_time_for_player(0).unwrap();
        assert!(elapsed.is_negative());

        let remain = timer.check_remaining_time_for_player(0).unwrap();
        assert_gt!(remain, test_maxtime_ms);
    }
}
