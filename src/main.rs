use rand::{seq::SliceRandom, thread_rng};
use std::fs::File;
use std::io::{prelude::*, stdin, BufRead};

const SHUFFLE_SIZE: usize = 156;
const RESHUFFLE: usize = 14;
const NUM_DECKS: usize = 6;
const BUST_KWD: &str = &"bust";
const STARTING_MONEY: usize = 1_000_000;
const NUM_LOOPS: usize = 100_000;
const BET_UNIT: usize = 5;

// --- CARDS ---

#[derive(Copy, Clone)]
enum Card {
    Def(u8),
    Maybe(u8, u8),
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Card::Def(n) => format!("{}", n).fmt(f),
            Card::Maybe(x, y) => format!("{} or {}", x, y).fmt(f),
        }
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Card::Def(n) => format!("{}", n).fmt(f),
            Card::Maybe(x, y) => format!("{} or {}", x, y).fmt(f),
        }
    }
}

impl std::ops::Add for Card {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::Def(a), Self::Def(b)) => Self::Def(a + b),
            (Self::Def(a), Self::Maybe(b, c)) | (Self::Maybe(b, c), Self::Def(a)) => {
                if a + c > 21 {
                    Self::Def(a + b)
                } else {
                    Self::Maybe(a + b, a + c)
                }
            }
            (Self::Maybe(a, b), _) => Self::Maybe(a + 1, b + 1),
        }
    }
}

impl std::ops::AddAssign for Card {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Def(a), Self::Def(b)) => a == b,
            (Self::Maybe(a, b), Self::Maybe(c, d)) => (a == c) && (b == d),
            _ => false,
        }
    }
}

macro_rules! real_sum {
    ( $e:expr ) => {
        match $e.sum {
            Card::Def(n) | Card::Maybe(_, n) => n,
        }
    };
}

#[derive(Clone)]
struct Cards {
    cards: Vec<Card>,
}

impl Cards {
    fn sum(&self) -> Card {
        let mut total = Card::Def(0);
        for card in self.cards.iter() {
            total += *card;
        }
        total
    }
}

// --- DECK ---

#[derive(Debug)]
struct Deck {
    cards: Vec<Card>,
    drawables: Vec<Card>,
    counter: isize,
}

impl Deck {
    // TODO: add counting mechanism to deck.draw()
    fn new() -> Self {
        let mut deck: Vec<Card> = (0..52 * NUM_DECKS).map(|_| Card::Def(1)).collect();
        const VALS: [Card; 13] = [
            Card::Maybe(1, 11),
            Card::Def(2),
            Card::Def(3),
            Card::Def(4),
            Card::Def(5),
            Card::Def(6),
            Card::Def(7),
            Card::Def(8),
            Card::Def(9),
            Card::Def(10),
            Card::Def(10),
            Card::Def(10),
            Card::Def(10),
        ];
        for i in 0..52 * NUM_DECKS {
            deck[i] = VALS[i % 13];
        }
        Deck {
            cards: deck,
            drawables: Vec::new(),
            counter: 0,
        }
    }

    fn shuffle(&mut self) {
        self.drawables = self
            .cards
            .choose_multiple(&mut thread_rng(), SHUFFLE_SIZE)
            .map(|&c| c)
            .collect();
        self.counter = 0;
    }

    fn draw(&mut self) -> Card {
        match self.drawables.pop() {
            Some(n) => {
                self.counter += match n {
                    Card::Maybe(_, _) | Card::Def(10) => 1,
                    Card::Def(d) if d < 7 => -1,
                    _ => 0,
                };
                n
            }
            None => panic!("Not enough cards!"),
        }
    }
}

// --- HAND ---

struct Hand {
    cards: Cards,
    sum: Card,
    busted: bool,
}

impl Hand {
    fn new(deck: &mut Deck) -> Self {
        let cards: Cards = Cards {
            cards: (0..2).map(|_| deck.draw()).collect(),
        };
        Hand {
            sum: cards.sum(),
            busted: false,
            cards,
        }
    }

    fn show(&self) {
        println!("{:?} = {}", self.cards.cards, self.sum);
    }

    fn hit(&mut self, deck: &mut Deck) {
        let card = deck.draw();
        self.cards.cards.push(card);
        self.sum += card;
    }
}

// --- MONEY ---

struct Wallet {
    balance: usize,
    bet: usize,
}

impl Wallet {
    fn new() -> Self {
        Wallet {
            balance: STARTING_MONEY,
            bet: 0,
        }
    }

    fn place_bet(&mut self, amount: usize) -> Result<(), ()> {
        if self.balance >= amount {
            self.balance -= amount;
            self.bet += amount;
            Ok(())
        } else {
            Err(())
        }
    }

    fn double(&mut self) -> Result<(), ()> {
        self.place_bet(self.bet)
    }

    fn pay_out(&mut self, multiplier: usize) {
        self.balance += multiplier * self.bet;
        self.bet = 0;
    }

    fn lose(&mut self) {
        self.bet = 0;
    }
}

// --- PROGRAM ---

fn choice(input: &str, deck: &mut Deck, hand: &mut Hand, wallet: &mut Wallet) -> bool {
    match input {
        "s" => return false,
        "h" => hand.hit(deck),
        "d" => {
            match wallet.double() {
                Ok(_) => {
                    hand.hit(deck);
                    hand.show();
                    return false;
                }
                Err(_) => {
                    println!("Balance too low (${})", wallet.balance);
                }
            };
        }
        BUST_KWD => return false,
        _ => (),
    }
    true
    // True means keep playing the round
}

#[allow(unused_assignments)]
fn play() {
    let mut wallet = Wallet::new();

    let mut deck = Deck::new();

    let mut running_balance: Vec<usize> = Vec::new();

    'play: for _ in 0..NUM_LOOPS {
        deck.shuffle();

        'main: loop {
            let remaining_cards = deck.drawables.len();
            if remaining_cards < RESHUFFLE {
                break 'main;
            }
            // println!("***********");
            // println!("Balance: ${}", wallet.balance);
            // println!("Place Bet:");

            // if deck.counter <= 2 {
            //     BET_UNIT
            // } else {
            //     let bet = BET_UNIT as f32
            //         * (((deck.counter * 52) as f32
            //             / (NUM_DECKS * 52 - SHUFFLE_SIZE + remaining_cards) as f32)
            //             - 1f32);
            //     if bet <= BET_UNIT as f32 {
            //         BET_UNIT
            //     } else {
            //         bet as usize
            //     }
            // }
            if let Err(_) = wallet.place_bet(BET_UNIT) {
                println!("Balance too low (${})", wallet.balance);
                break 'main;
            }

            let mut hand = Hand::new(&mut deck);

            if hand.sum == Card::Maybe(11, 21) {
                // hand.show();
                // println!("Blackjack!");
                wallet.pay_out(3);
                continue 'main;
            }

            let mut dealer = Hand::new(&mut deck);
            // println!("Dealer: {}", dealer.cards.cards[0]);

            // --- Inputs ---

            let mut split = false;
            let mut bet = 0usize;
            let mut hand2 = Hand {
                cards: Cards { cards: vec![] },
                busted: false,
                sum: Card::Def(0),
            };

            macro_rules! player_input {
                ( $e:expr ) => {{
                    $e.show();
                    if match $e.sum {
                        Card::Def(n) => n > 21,
                        _ => false,
                    } {
                        println!("Busted!");
                        $e.busted = true;
                        BUST_KWD
                    } else {
                        match &stdin().lock().lines().next().unwrap().unwrap()[..] {
                            "s" => "s",
                            "h" => "h",
                            "d" => "d",
                            "sp" => {
                                if $e.cards.cards[0] == $e.cards.cards[1]
                                    && wallet.balance >= wallet.bet
                                    && bet == 0
                                {
                                    split = true;
                                    $e.cards.cards.remove(1);
                                    $e.sum = $e.cards.cards[0];
                                    "h"
                                } else {
                                    "tortoise"
                                }
                            }
                            "q" => break 'play,
                            _ => "tortoise",
                        }
                    }
                }};
            }

            macro_rules! dealer_input {
                () => {{
                    if real_sum!(dealer) < 17 {
                        "h"
                    } else {
                        if match dealer.sum {
                            Card::Def(n) => n > 21,
                            _ => false,
                        } {
                            dealer.busted = true;
                            BUST_KWD
                        } else {
                            "s"
                        }
                    }
                }};
            }

            macro_rules! basic_input {
                () => {{
                    if real_sum!(hand) < 17 {
                        "h"
                    } else {
                        if match hand.sum {
                            Card::Def(n) => n > 21,
                            _ => false,
                        } {
                            hand.busted = true;
                            BUST_KWD
                        } else {
                            "s"
                        }
                    }
                }};
            }

            // --- Success Validation ---

            macro_rules! win_lose {
                ( $e:expr ) => {
                    let hand_final = real_sum!($e);
                    let dealer_final = real_sum!(dealer);
                    if !$e.busted && (hand_final >= dealer_final || dealer.busted) {
                        if hand_final == dealer_final {
                            // println!("Push!");
                            wallet.pay_out(1);
                        } else {
                            // println!("You Win!");
                            wallet.pay_out(2);
                        }
                    } else {
                        // println!("You Lose!");
                        wallet.lose();
                    }
                };
            }

            // --- Play ---
            while choice(dealer_input!(), &mut deck, &mut dealer, &mut wallet) {}

            while choice(basic_input!(), &mut deck, &mut hand, &mut wallet) {
                if split {
                    bet = wallet.bet;
                    let val = hand.cards.cards[0] + Card::Def(0);
                    hand2 = Hand {
                        cards: Cards { cards: vec![val] },
                        busted: false,
                        sum: val,
                    };
                    hand2.hit(&mut deck);

                    // println!("--- HAND 1 ---");
                    while choice(player_input!(hand2), &mut deck, &mut hand2, &mut wallet) {}

                    // println!("--- HAND 2 ---");
                    split = false;
                }
            }

            // hand.show();
            // println!("--- DEALER ---");
            // dealer.show();

            if bet != 0 {
                win_lose!(hand2);
                wallet.place_bet(bet).unwrap();
            }

            win_lose!(hand);
            running_balance.push(wallet.balance);
        }

        // println!("Reshuffling cards...");
    }

    println!("Final Balance: {}", wallet.balance);
    // let mut file = File::create("tmp/without_bet.txt").unwrap();
    // writeln!(
    //     file,
    //     "{}",
    //     running_balance
    //         .iter()
    //         .map(|w| w.to_string())
    //         .collect::<Vec<String>>()
    //         .join(", ")
    // )
    // .unwrap();
}

fn main() {
    play();
}
