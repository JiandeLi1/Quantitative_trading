use std::thread;
use ringbuf::RingBuffer;
use core_affinity;
use std::time::{Duration, SystemTime};


#[derive(Debug)]
pub enum Side { Bid, Ask }

#[derive(Debug)]
pub struct L2Level {
    pub price: f64,
    pub qty: f64,
    pub count: u32,
}

#[derive(Debug)]
pub struct L2BookUpdates{
    pub sequence_id: u64,
    pub timestamp: u64,
    pub instrument_id: u32,
    pub bids : Vec<L2Level>,
    pub asks : Vec<L2Level>,
}

#[derive(Debug)]
pub struct TradeUpdate {
    pub sequence_id: u64,
    pub timestamp: u64,
    pub instrument_id: u32,

    pub side: Side,
    pub price : f64,
    pub qty: f64,
}
impl TradeUpdate{
    fn new (s_id: u64, time:u64, i_id:u32, s: Side, p:f64, q:f64)->TradeUpdate{
        TradeUpdate{
            sequence_id:s_id,
            timestamp:time,
            instrument_id:i_id,
            side:s,
            price:p,
            qty:q
        }
    }
}

pub enum MarketDataMessage {
    Quote(L2BookUpdates),
    Trade(TradeUpdate),
    Close
}

fn main(){
    //A ring buffer is used as storage space for buffering input or output data.
    let rb = RingBuffer::<MarketDataMessage>::new(100);

    //producer from ring buffer is used for adding data to buffer, consumer is used to pop the data from ring buffer
    let (mut prod, mut cons) = rb.split();

    //The working cores from your computer
    let core_ids = core_affinity::get_core_ids().unwrap();
    let core_0=core_ids[0];//core 0
    let core_1=core_ids[1];//core 1
    // producer
    let pjh = thread::spawn(move || {
        // set only core_0 work for the producer 
        core_affinity::set_for_current(core_0);

        // push some data
        for i in 0..10 {
            let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let ts_mics = u64::try_from(ts.as_micros()).unwrap();
            let update = TradeUpdate::new(i, ts_mics, 0, Side::Bid, 1000.0, 10.0);
            prod.push(MarketDataMessage::Trade(update));
        }

        // tell the consumer it can exit
        prod.push(MarketDataMessage::Close);
    });
    // consumer
    let cjh = thread::spawn(move || {
        // set only core_1 work for the consumer 
        core_affinity::set_for_current(core_1);

        loop {
            if !cons.is_empty() {  // this is basically using a spin lock
                match cons.pop().unwrap() {
                    MarketDataMessage::Quote(book_update) => {
                        println!("{:?}", book_update);
                    }
                    MarketDataMessage::Trade(trade_update) => {
                        println!("{:?}", trade_update);
                    }
                    MarketDataMessage::Close => {
                        break;
                    }
                }
            }
        }
    });

    // wait for threads to complete
    pjh.join().unwrap();
    cjh.join().unwrap();
}