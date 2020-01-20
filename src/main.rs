use rand::{seq::SliceRandom, thread_rng};
use std::io::{stdin, BufRead};

const SHUFFLE_SIZE: usize = 62;
const RESHUFFLE: usize = 10;
const NUM_DECKS: usize = 4;
const BUST_KWD: &str = &"bust";
const STARTING_MONEY: usize = 1_000;

// --- DECK ---

#[derive(Debug)]
struct Deck {
    cards: Vec<u8>,
    drawables: Vec<u8>,
}

impl Deck {
    // TODO: add counting mechanism to deck.draw()
    fn new() -> Self {
        let mut deck: Vec<u8> = (0..52 * NUM_DECKS as u8).collect();
        const VALS: [u8; 13] = [11, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10];
        for i in 0..52 * NUM_DECKS {
            deck[i] = VALS[i % 13];
        }
        Deck {
            cards: deck,
            drawables: Vec::new(),
        }
    }

    fn shuffle(&mut self) {
        self.drawables = self
            .cards
            .choose_multiple(&mut thread_rng(), SHUFFLE_SIZE)
            .cloned()
            .collect();
        // println!("{:?}", self.drawables);
        // let (draws, _) = self.cards.partial_shuffle(&mut thread_rng(), SHUFFLE_SIZE);
        // self.drawables = (*draws).to_vec();
    }

    fn draw(&mut self) -> u8 {
        match self.drawables.pop() {
            Some(n) => n,
            None => panic!("Not enough cards!"),
        }
    }
}

// --- HAND ---

struct Hand {
    cards: Vec<u8>,
    sum: u8,
    busted: bool,
}

impl Hand {
    fn new(draw_size: u8, deck: &mut Deck) -> Self {
        let cards: Vec<u8> = (0..draw_size).map(|_| deck.draw()).collect();
        Hand {
            sum: cards.iter().sum(),
            busted: false,
            cards,
        }
    }

    fn show(&self) {
        println!("{:?} = {}", self.cards, self.sum);
    }

    fn hit(&mut self, deck: &mut Deck) {
        let card = deck.draw();
        self.cards.push(card);
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
                    return true;
                }
            };
        }
        BUST_KWD => return false,
        _ => (),
    }
    true
    // True means keep playing the round
}

fn play() {
    let mut wallet = Wallet::new();

    let mut deck = Deck::new();
    deck.shuffle();

    'main: loop {
        if deck.drawables.len() < RESHUFFLE {
            break 'main;
        }
        println!("-------");
        println!("Balance: ${}", wallet.balance);
        println!("Place Bet:");
        if let Err(_) = wallet.place_bet(loop {
            if let Ok(n) = stdin()
                .lock()
                .lines()
                .next()
                .unwrap()
                .unwrap()
                .parse::<usize>()
            {
                break n;
            }
        }) {
            println!("Balance too low (${})", wallet.balance);
            continue 'main;
        }

        let mut hand = Hand::new(2, &mut deck);

        if hand.sum == 21 {
            hand.show();
            println!("Blackjack!");
            wallet.pay_out(3);
            continue 'main;
        }

        let mut dealer = Hand::new(2, &mut deck);
        println!("Dealer: {}", dealer.cards[0]);

        while choice(
            // Player
            {
                hand.show();
                if hand.sum > 21 {
                    println!("Busted!");
                    hand.busted = true;
                    BUST_KWD
                } else {
                    match &stdin().lock().lines().next().unwrap().unwrap()[..] {
                        "s" => "s",
                        "h" => "h",
                        "d" => "d",
                        _ => "tortoise",
                    }
                }
            },
            &mut deck,
            &mut hand,
            &mut wallet,
        ) {}

        println!("--- DEALER ---");

        while choice(
            // Dealer
            {
                if dealer.sum < 17 {
                    "h"
                } else {
                    dealer.show();
                    if dealer.sum > 21 {
                        dealer.busted = true;
                        BUST_KWD
                    } else {
                        "s"
                    }
                }
            },
            &mut deck,
            &mut dealer,
            &mut wallet,
        ) {}

        if !hand.busted && (hand.sum >= dealer.sum || dealer.busted) {
            if hand.sum == dealer.sum {
                println!("Push!");
                wallet.pay_out(1);
            } else {
                println!("You Win!");
                wallet.pay_out(2);
            }
        } else {
            println!("You Lose!");
            wallet.lose();
        }
    }
    println!("No cards left!");
}

fn main() {
    play();
}
