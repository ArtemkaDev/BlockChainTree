#![allow(non_snake_case)]
use num_bigint::BigUint;
use sha2::{Sha256, Digest};
use std::convert::TryInto;
use crate::Tools;
use crate::Transaction;
use crate::Token;
use crate::Block::{TransactionBlock, TokenBlock, TransactionToken};
use std::mem::transmute_copy;
use std::collections::VecDeque;

use std::env;
use std::fs;
use std::io;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::collections::HashMap;
use std::str;
use std::path::Path;
use rocksdb::{DBWithThreadMode as DB, Options, MultiThreaded};


static BLOCKS_DIRECTORY:&'static str = "./BlockChainTree/";

static AMMOUNT_SUMMARY:&'static str = "./BlockChainTree/SUMMARY/";

static MAIN_CHAIN_DIRECTORY:&'static str = "./BlockChainTree/MAIN/";


static DERIVATIVE_CHAINS_DIRECTORY:&'static str = "./BlockChainTree/DERIVATIVE/CHAINS/";
static DERIVATIVE_DB_DIRECTORY:&'static str = "./BlockChainTree/DERIVATIVE/DB/";

static BLOCKS_FOLDER:&'static str = "BLOCKS/";
static REFERENCES_FOLDER:&'static str = "REF/";

static CONFIG_FILE:&'static str = "Chain.config";
static LOOKUP_TABLE_FILE:&'static str = "LookUpTable.dat"; 
static GENESIS_BLOCK:[u8;32] = [0x77,0xe6,0xd9,0x52,
                                0x67,0x57,0x8e,0x85,
                                0x39,0xa9,0xcf,0xe0,
                                0x03,0xf4,0xf7,0xfe,
                                0x7d,0x6a,0x29,0x0d,
                                0xaf,0xa7,0x73,0xa6,
                                0x5c,0x0f,0x01,0x9d,
                                0x5c,0xbc,0x0a,0x7c];
static BEGINNING_DIFFICULTY:[u8;32] = [0x7F,0xFF,0xFF,0xFF,
                                       0xFF,0xFF,0xFF,0xFF,
                                       0xFF,0xFF,0xFF,0xFF,
                                       0xFF,0xFF,0xFF,0xFF,
                                       0xFF,0xFF,0xFF,0xFF,
                                       0xFF,0xFF,0xFF,0xFF,
                                       0xFF,0xFF,0xFF,0xFF,
                                       0xFF,0xFF,0xFF,0xFF];
// God is dead, noone will stop anarchy

static MAX_TRANSACTIONS_PER_BLOCK:usize = 3000;


pub struct Chain{
    db: DB::<MultiThreaded>,
    height_reference: DB::<MultiThreaded>,
    height:u64,
    genesis_hash:[u8;32],
    difficulty:[u8;32]
}

impl Chain{
    pub fn new(root_path:&str) -> Result<Chain,&'static str>{
        let root = String::from(root_path);
        let path_blocks_st = root.clone() + BLOCKS_FOLDER;
        let path_references_st = root.clone() + REFERENCES_FOLDER;
        let path_height_st = root+CONFIG_FILE;

        let path_blocks = Path::new(&path_blocks_st);
        let path_reference = Path::new(&path_references_st);
        let path_height = Path::new(&path_height_st);

        // open blocks DB
        let result = DB::<MultiThreaded>::open_default(
                                            path_blocks);
        if result.is_err(){
            return Err("Error opening blocks db");
        }
        let db = result.unwrap();

        // open height references DB
        let result = DB::<MultiThreaded>::open_default(
                                            path_reference);
        if result.is_err(){
            return Err("Error opening references db");
        }
        let references = result.unwrap();
        
        
        let result = File::open(path_height);
        if result.is_err(){
            return Err("Could not open config");
        }
        let mut file = result.unwrap();

        // read height from config
        let mut height_bytes:[u8;8] = [0;8];
        let result = file.read_exact(&mut height_bytes);
        if result.is_err(){
            return Err("Error reading config");
        }
        let height:u64 = u64::from_be_bytes(height_bytes);

        // read genesis hash
        let mut genesis_hash:[u8;32] = [0;32];
        let result = file.read_exact(&mut genesis_hash);
        if result.is_err(){
            return Err("Error reading genesis hash from config");
        }

        // read difficulty
        let mut difficulty:[u8;32] = [0;32];
        let result = file.read_exact(&mut difficulty);
        if result.is_err(){
            return Err("Error reading diffculty from config");
        }

        return Ok(Chain{db:db,
                height_reference:references,
                height:height,
                genesis_hash:genesis_hash,
                difficulty:difficulty});
    }

    pub fn add_block(&mut self,
                    block:&TransactionBlock) -> Result<(),&'static str>{

        let result = block.dump();
        if result.is_err(){
            return Err(result.err().unwrap());
        }
        let dump = result.unwrap();

        let hash = Tools::hash(&dump);


        let result = self.db.put(hash,dump);
        if result.is_err(){
            return Err("Error adding block");
        }

        let result = self.height_reference.put(self.height.to_be_bytes(),
                                                hash);
        if result.is_err(){
            return Err("Error adding reference");
        }

        self.height += 1;

        return Ok(());
    }

    pub fn get_height(&self) -> u64{
        return self.height;
    }

    pub fn get_difficulty(&self) -> [u8;32]{
        return self.difficulty;
    }

    pub fn find_by_height(&self,height:u64) -> Result<Option<TransactionBlock>,&'static str>{
        if height > self.height{
            return Ok(None);
        }
        let result = self.db.get(height.to_be_bytes());
        if result.is_err(){
            return Err("Error reading block");
        }
        let result = result.unwrap();
        if result.is_none(){
            return Ok(None);
        }
        let dump = result.unwrap();

        let result = TransactionBlock::parse(&dump,dump.len() as u32);
        if result.is_err(){
            return Err(result.err().unwrap());
        }
        let block = result.unwrap();
        return Ok(Some(block));

    }

    pub fn find_by_hash(&self,hash:&[u8;32]) -> Result<Option<TransactionBlock>,&'static str>{
        let result = self.height_reference.get(hash);
        if result.is_err(){
            return Err("Error getting height");
        }
        let result = result.unwrap();
        if result.is_none(){
            return Ok(None);
        }
        let height = u64::from_be_bytes(result.unwrap().try_into().unwrap());

        let result = self.find_by_height(height);
        if result.is_err(){
            return Err(result.err().unwrap());
        }
        let block = result.unwrap();

        return Ok(block);

    }

    pub fn dump_config(&self, root_path:&str) -> Result<(),&'static str>{
        let root = String::from(root_path);
        let path_config = root+CONFIG_FILE;

        let result = File::create(path_config);
        if result.is_err(){
            return Err("Could not open config");
        }
        let mut file = result.unwrap();

        let result = file.write_all(
                            &self.height.to_be_bytes());
        if result.is_err(){
            return Err("Error writing height");
        }

        let result = file.write_all(&self.genesis_hash);
        if result.is_err(){
            return Err("Error writing genesis block");
        }

        let result = file.write_all(&self.difficulty);
        if result.is_err(){
            return Err("Error writing difficulty");
        }

        return Ok(())
    }

    pub fn new_without_config(root_path:&str,
                            genesis_hash:&[u8;32]) -> Result<Chain,&'static str>{
        let root = String::from(root_path);
        let path_blocks_st = root.clone() + BLOCKS_FOLDER;
        let path_references_st = root.clone() + REFERENCES_FOLDER;
        
        let path_blocks = Path::new(&path_blocks_st);
        let path_reference = Path::new(&path_references_st);
    
        // open blocks DB
        let result = DB::<MultiThreaded>::open_default(
            path_blocks);
        if result.is_err(){
            return Err("Error opening blocks db");
        }
        let db = result.unwrap();

        // open height references DB
        let result = DB::<MultiThreaded>::open_default(
                    path_reference);
        if result.is_err(){
            return Err("Error opening references db");
        }
        let references = result.unwrap();

        return Ok(Chain{db:db,
                        height_reference:references,
                        height:0,
                        genesis_hash:*genesis_hash,
                        difficulty:BEGINNING_DIFFICULTY});
    }

}

pub struct DerivativeChain{
    db: DB::<MultiThreaded>,
    height_reference: DB::<MultiThreaded>,
    height:u64,
    global_height:u64,
    genesis_hash:[u8;32],
    difficulty:[u8;32]
}

impl DerivativeChain{
    pub fn new(root_path:&str) -> Result<DerivativeChain,&'static str>{
        let root = String::from(root_path);
        let path_blocks_st = root.clone() + BLOCKS_FOLDER;
        let path_references_st = root.clone() + REFERENCES_FOLDER;
        let path_height_st = root+CONFIG_FILE;

        let path_blocks = Path::new(&path_blocks_st);
        let path_reference = Path::new(&path_references_st);
        let path_height = Path::new(&path_height_st);

        // open blocks DB
        let result = DB::<MultiThreaded>::open_default(
                                            path_blocks);
        if result.is_err(){
            return Err("Error opening blocks db");
        }
        let db = result.unwrap();

        // open height references DB
        let result = DB::<MultiThreaded>::open_default(
                                            path_reference);
        if result.is_err(){
            return Err("Error opening references db");
        }
        let references = result.unwrap();
        
        
        let result = File::open(path_height);
        if result.is_err(){
            return Err("Could not open config");
        }
        let mut file = result.unwrap();

        // read height from config
        let mut height_bytes:[u8;8] = [0;8];
        let result = file.read_exact(&mut height_bytes);
        if result.is_err(){
            return Err("Error reading config");
        }
        let height:u64 = u64::from_be_bytes(height_bytes);

        // read genesis hash
        let mut genesis_hash:[u8;32] = [0;32];
        let result = file.read_exact(&mut genesis_hash);
        if result.is_err(){
            return Err("Error reading genesis hash from config");
        }

        // read difficulty
        let mut difficulty:[u8;32] = [0;32];
        let result = file.read_exact(&mut difficulty);
        if result.is_err(){
            return Err("Error reading diffculty from config");
        }

        // read global height
        let mut global_height:[u8;8] = [0;8];
        let result = file.read_exact(&mut global_height);
        if result.is_err(){
            return Err("Error reading global height from config");
        }
        let global_height:u64 = u64::from_be_bytes(global_height);

        return Ok(DerivativeChain{db:db,
                height_reference:references,
                height:height,
                genesis_hash:genesis_hash,
                difficulty:difficulty,
                global_height:global_height});
    }

    pub fn add_block(&mut self,
                    block:&TokenBlock) -> Result<(),&'static str>{

        let result = block.dump();
        if result.is_err(){
            return Err(result.err().unwrap());
        }
        let dump = result.unwrap();

        let hash = Tools::hash(&dump);


        let result = self.db.put(hash,dump);
        if result.is_err(){
            return Err("Error adding block");
        }

        let result = self.height_reference.put(self.height.to_be_bytes(),
                                                hash);
        if result.is_err(){
            return Err("Error adding reference");
        }

        self.height += 1;

        return Ok(());
    }

    pub fn get_height(&self) -> u64{
        return self.height;
    }

    pub fn get_difficulty(&self) -> [u8;32]{
        return self.difficulty;
    }

    pub fn get_global_height(&self) -> u64{
        return self.global_height;
    }

    pub fn find_by_height(&self,height:u64) -> Result<Option<TokenBlock>,&'static str>{
        if height > self.height{
            return Ok(None);
        }
        let result = self.db.get(height.to_be_bytes());
        if result.is_err(){
            return Err("Error reading block");
        }
        let result = result.unwrap();
        if result.is_none(){
            return Ok(None);
        }
        let dump = result.unwrap();

        let result = TokenBlock::parse(&dump,dump.len() as u32);
        if result.is_err(){
            return Err(result.err().unwrap());
        }
        let block = result.unwrap();
        return Ok(Some(block));

    }

    pub fn find_by_hash(&self,hash:&[u8;32]) -> Result<Option<TokenBlock>,&'static str>{
        let result = self.height_reference.get(hash);
        if result.is_err(){
            return Err("Error getting height");
        }
        let result = result.unwrap();
        if result.is_none(){
            return Ok(None);
        }
        let height = u64::from_be_bytes(result.unwrap().try_into().unwrap());

        let result = self.find_by_height(height);
        if result.is_err(){
            return Err(result.err().unwrap());
        }
        let block = result.unwrap();

        return Ok(block);

    }

    pub fn dump_config(&self, root_path:&str) -> Result<(),&'static str>{
        let root = String::from(root_path);
        let path_config = root+CONFIG_FILE;

        let result = File::create(path_config);
        if result.is_err(){
            return Err("Could not open config");
        }
        let mut file = result.unwrap();

        let result = file.write_all(
                            &self.height.to_be_bytes());
        if result.is_err(){
            return Err("Error writing height");
        }

        let result = file.write_all(&self.genesis_hash);
        if result.is_err(){
            return Err("Error writing genesis block");
        }

        let result = file.write_all(&self.difficulty);
        if result.is_err(){
            return Err("Error writing difficulty");
        }

        let result = file.write_all(
                        &self.global_height.to_be_bytes());
        if result.is_err(){
            return Err("Error writing global height");
        }

        return Ok(())
    }

    pub fn new_without_config(root_path:&str,
                            genesis_hash:&[u8;32],
                            global_height:u64) -> Result<DerivativeChain,&'static str>{
        let root = String::from(root_path);
        let path_blocks_st = root.clone() + BLOCKS_FOLDER;
        let path_references_st = root.clone() + REFERENCES_FOLDER;
        
        let path_blocks = Path::new(&path_blocks_st);
        let path_reference = Path::new(&path_references_st);
    
        // open blocks DB
        let result = DB::<MultiThreaded>::open_default(
            path_blocks);
        if result.is_err(){
            return Err("Error opening blocks db");
        }
        let db = result.unwrap();

        // open height references DB
        let result = DB::<MultiThreaded>::open_default(
                    path_reference);
        if result.is_err(){
            return Err("Error opening references db");
        }
        let references = result.unwrap();

        return Ok(DerivativeChain{db:db,
                        height_reference:references,
                        height:0,
                        genesis_hash:*genesis_hash,
                        difficulty:BEGINNING_DIFFICULTY,
                        global_height:global_height});
    }

}


pub struct BlockChainTree{
    transactions_pool: VecDeque<TransactionToken>,
    summary_db: DB::<MultiThreaded>,
    main_chain:Chain,
    opened_derivatives:HashMap<[u8;32],(u32,DerivativeChain)>

}


impl BlockChainTree{

}

