#[ink::contract]
pub mod number_guessing_game {
    use ink::env::AccountId;

    #[ink(storage)]
    pub struct NumberGuessingGame {
        secret_number: u8,      // The number to guess (0-9)
        prize_pool: Balance,    // Total prize available to win
        guess_fee: Balance,     // Fee per guess (e.g., 1 token)
        owner: AccountId,       // Contract deployer who sets the number
    }

    impl NumberGuessingGame {
        /// Constructor: Sets the secret number and initializes the prize pool with sent funds
        #[ink(constructor, payable)]
        pub fn new(secret_number: u8) -> Self {
            let initial_prize = Self::env().transferred_value();
            Self {
                secret_number,
                prize_pool: initial_prize,
                guess_fee: 1_000_000_000_000, // 1 token, assuming 12 decimal places
                owner: Self::env().caller(),
            }
        }

        /// Players guess the number by paying the guess fee
        #[ink(message, payable)]
        pub fn guess(&mut self, number: u8) -> bool {
            let paid = self.env().transferred_value();
            if paid < self.guess_fee || self.prize_pool == 0 {
                return false; // Not enough paid or game is over
            }

            if number == self.secret_number {
                // Winner! Transfer the entire prize pool
                self.env().transfer(self.env().caller(), self.prize_pool).expect("Transfer failed");
                self.prize_pool = 0; // Game ends
                true
            } else {
                // Incorrect guess: Add the fee to the prize pool
                self.prize_pool += self.guess_fee;
                false
            }
        }

        /// Check the current prize pool
        #[ink(message)]
        pub fn get_prize_pool(&self) -> Balance {
            self.prize_pool
        }

        /// Check the guess fee
        #[ink(message)]
        pub fn get_guess_fee(&self) -> Balance {
            self.guess_fee
        }

        /// Owner can reset the game with a new secret number and optional additional funds
        #[ink(message, payable)]
        pub fn reset_game(&mut self, new_secret_number: u8) {
            assert_eq!(self.env().caller(), self.owner, "Only owner can reset the game");
            self.secret_number = new_secret_number;
            let additional_funds = self.env().transferred_value();
            self.prize_pool += additional_funds;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn guess_correctly_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            let initial_prize = 10_000_000_000_000; // 10 tokens
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(initial_prize);
            let mut game = NumberGuessingGame::new(5);

            // Player guesses correctly
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(game.guess_fee);
            let result = game.guess(5);
            assert!(result);
            assert_eq!(game.get_prize_pool(), 0);
        }

        #[ink::test]
        fn guess_incorrectly_increases_prize() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            let initial_prize = 10_000_000_000_000; // 10 tokens
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(initial_prize);
            let mut game = NumberGuessingGame::new(5);

            // Player guesses incorrectly
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(game.guess_fee);
            let result = game.guess(3);
            assert!(!result);
            assert_eq!(game.get_prize_pool(), initial_prize + game.guess_fee);
        }
    }
}