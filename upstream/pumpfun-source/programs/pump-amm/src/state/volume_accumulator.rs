use crate::*;

// Global token incentives schedule.
//
// One account per incentive mint, seeded with `global_volume_accumulator`.
// Tracks, for each of the 30 days of the incentive range, the token supply
// that will be distributed and the total SOL volume traded on that day.
#[account]
#[derive(Debug)]
pub struct GlobalVolumeAccumulator {
    pub start_time:i64,
    pub end_time:i64,
    pub seconds_in_a_day:i64,
    pub mint:Pubkey,
    pub total_token_supply:[u64;30],
    pub sol_volumes:[u64;30],
}

// Per-user volume tracking used to compute token incentives and cashback.
//
// Created lazily on the first trade of a user and updated on every buy and
// sell. Unclaimed tokens are moved to `total_unclaimed_tokens` when a day
// rolls over, and are paid out by `claim_token_incentives`.
// Cashback is accumulated separately and paid out by `claim_cashback`.
#[account]
pub struct UserVolumeAccumulator {
    pub user: Pubkey,
    pub needs_claim: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
    pub has_total_claimed_tokens: bool,
    pub cashback_earned: u64,
    pub total_cashback_claimed: u64,
}

//
//
//
//
//
//
//
//
//
//
impl UserVolumeAccumulator {
    // Adds `sol_volume` to the volume traded by the user during the current day.
    fn add_sol_volume(&mut self, sol_volume: u64, now: i64) {
        self.current_sol_volume = self
            .current_sol_volume
            .checked_add(sol_volume)
            .expect("Overflowing user accumulator sol volume");
        self.needs_claim = self.total_unclaimed_tokens > 0 || self.current_sol_volume > 0;
        self.last_update_timestamp = now;
    }
}
//
//
//
//
//

impl GlobalVolumeAccumulator {
    //
    //
    //
    //
    //
    //
    //
    //
    fn roll_over(&self, user: &mut UserVolumeAccumulator, current_day: usize, now: i64) -> Result<()> {
        if let Some(last_update_day) = self.day_index(user.last_update_timestamp) {
            let end_day = self.end_day();
            if last_update_day < current_day && last_update_day <= end_day {
                let sol_volume =
                self.sol_volumes[last_update_day];
                let token_supply = self.total_token_supply[last_update_day];
                //
                //
                //
                //
                //
                //
                //
                //
                //
                //
                //
                //
                //
                //
                //
                //
                if sol_volume > 0 {
                if user.current_sol_volume == 0 {
                    return Ok(());
                }
                let tokens = (user.current_sol_volume as u128) * (token_supply as u128)
                    / (sol_volume as u128);
                let Ok(tokens) = u64::try_from(tokens) else {
                    return err!(PumpAmmError::Truncation);
                };
        user.total_unclaimed_tokens = user
            .total_unclaimed_tokens
            .checked_add(tokens)
            .expect("Unexpected overflow");
                user.needs_claim = user.total_unclaimed_tokens > 0;
                user.last_update_timestamp = now;
                user.current_sol_volume = 0;
                }
            }
        }
        Ok(())
    }
}

//
//
//
//
//
//
//
//
//

impl UserVolumeAccumulator {
    // Moves every unclaimed token to the claimed total after a successful
    // `claim_token_incentives` transfer.
    pub fn claim_tokens(&mut self, now: i64) {
        self.total_claimed_tokens = self
            .total_claimed_tokens
            .checked_add(self.total_unclaimed_tokens)
            .unwrap();
        self.total_unclaimed_tokens = 0;
        self.has_total_claimed_tokens = self.total_claimed_tokens > 0;
        self.needs_claim = self.current_sol_volume > 0;
        self.last_update_timestamp = now;
    }
}

impl GlobalVolumeAccumulator {
    // Rolls the user's volume of a finished day into unclaimed tokens and
    // records `sol_volume` for the current day of the incentive range.
    pub fn update_user_volume_accumulator(
        &mut self,
        user: &mut UserVolumeAccumulator,
        now: i64,
        sol_volume: u64,
    ) -> Result<()> {
        if user.last_update_timestamp < self.start_time && now < self.start_time {
            user.total_claimed_tokens = 0;
            user.has_total_claimed_tokens = false;
        }
        let result = match self.day_index(now) {
            Some(current_day) => self.roll_over(user, current_day, now).map(|()| Some(current_day)),
            None => return Ok(()),
        };
        if let Ok(Some(current_day)) = result {
            if now <= self.end_time {
                self.add_sol_volume(current_day, sol_volume);
                user.add_sol_volume(sol_volume, now);
            }
        }
        result.map(|_| ())
    }

    /// Index of the incentive day containing `timestamp`, or `None` before the
    /// start of the incentive range or when no range has been configured.
    fn day_index(&self, timestamp: i64) -> Option<usize> {
        to_usize(timestamp)
            .checked_sub(to_usize(self.start_time))
            .and_then(|value| {
                (self.seconds_in_a_day != 0).then(|| value / to_usize(self.seconds_in_a_day))
            })
    }

    fn update_token_supply(
        &mut self,
        day_number: u64,
        token_supply_per_day: u64,
        current_timestamp: i64,
    ) -> Result<()> {
        if current_timestamp < self.start_time {
            return self.set_token_supply_before_start(day_number, token_supply_per_day);
        }
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        //
        if current_timestamp > self.end_time {
            return err!(PumpAmmError::SupplyUpdateForFinishedRange);
        }
        //
        //
        self.set_token_supply_after_start(day_number, token_supply_per_day, current_timestamp)
    }
    fn set_token_supply_before_start(&mut self, day_number: u64, token_supply_per_day: u64) -> Result<()> {
        let end_day = self.end_day();
        if day_number < 30 {
        //
        //
        //
            require_gte!(end_day, day_number as usize, PumpAmmError::DayIndexAfterEndIndex);
            self.total_token_supply[day_number as usize] = token_supply_per_day;
            return Ok(());
        }
        //
        //
        //
        for day in 0..=end_day {
            self.total_token_supply[day] = token_supply_per_day;
        }
        Ok(())
    }

    fn set_token_supply_after_start(
        &mut self,
        day_number: u64,
        token_supply_per_day: u64,
        current_timestamp: i64,
    ) -> Result<()> {
        //
        let current_day = self
            .day_index(current_timestamp)
            .expect("current_timestamp should be bigger than start_time");
        let end_day = self.end_day();
        if day_number < 30 {
        //
        //
        //
            require_gte!(end_day, day_number as usize, PumpAmmError::DayIndexAfterEndIndex);
            require_gte!(day_number as usize, current_day, PumpAmmError::DayInActiveRange);
            self.total_token_supply[day_number as usize] = token_supply_per_day;
            return Ok(());
        }
        //
        //
        //
        for day in current_day..=end_day {
            self.total_token_supply[day] = token_supply_per_day;
        }
        Ok(())
    }

    fn update_time_range(&mut self, start_time: i64, end_time: i64, current_timestamp: i64) -> Result<()> {
        if start_time != self.start_time {
            //
            //
            //
            //
            //
            //
            require_gt!(start_time, current_timestamp, PumpAmmError::StartTimeInThePast);
            self.start_time = start_time;
            self.total_token_supply = [0; 30];
            self.sol_volumes = [0; 30];
        }
        //
        //
        //
        //
        //
        //
        //
        //
        //
        require_gt!(end_time, current_timestamp, PumpAmmError::EndTimeInThePast);
        //
        require_gt!(end_time, start_time, PumpAmmError::EndTimeBeforeStartTime);
        //
        require_gte!(
            self.seconds_in_a_day
                .checked_mul(30)
                .expect("Unexpected overflow")
                - 1,
            //
            end_time
                .checked_sub(start_time)
                .expect("Unexpected overflow"),
            PumpAmmError::TimeRangeTooLarge
        );
        self.end_time = end_time;
        if current_timestamp > self.start_time {
            //
            //
            let current_day = self
                .day_index(current_timestamp)
                .expect("current_timestamp should be bigger than start_time");
            let end_day = self.end_day();
            let next_day = current_day + 1;
            //
            require_gt!(end_day, current_day, PumpAmmError::EndTimeBeforeCurrentDay);
            //
            //
            //
            //
            //
            for day in next_day..=end_day {
                self.sol_volumes[day] = 0;
            }
        }
        Ok(())
    }

    pub fn update_token_incentives(
        &mut self,
        start_time: i64, end_time: i64, seconds_in_a_day: i64,
        mint: &Pubkey,
        day_number: u64, token_supply_per_day: u64, current_timestamp: i64,
    ) -> Result<()> {
        self.seconds_in_a_day = seconds_in_a_day;
        self.mint = *mint;
        if start_time != 0 && end_time != 0 {
            self.update_time_range(start_time, end_time, current_timestamp)?;
        }
        if token_supply_per_day != 0 {
            self.update_token_supply(day_number, token_supply_per_day, current_timestamp)?;
        }

        Ok(())
    }

    pub fn end_day(&self)->usize {
        to_usize(self.end_time)
            .checked_sub(to_usize(self.start_time))
            .and_then(|value| (self.seconds_in_a_day != 0).then(|| value / to_usize(self.seconds_in_a_day)))
            .expect("end_time should be bigger than start_time")
    }

    fn add_sol_volume(&mut self, current_index: usize, sol_volume: u64) {
        self.sol_volumes[current_index] = self.sol_volumes[current_index]
            .checked_add(sol_volume)
            .expect("Overflowing sol volume");
    }
}

fn to_usize(value: i64) -> usize {
    usize::try_from(value).expect("SVM is 64-bit")
}
