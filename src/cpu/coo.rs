use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;

pub struct COO {
    pub data: Vec<(usize, usize, f64)>,
    pub shape: (usize, usize)
}

impl COO {

    // Read mtx-file, its honestly almost the same thing as
    // in the last homeworks, so I didn't comment too much
    pub fn read_mtx(fname: &str) -> Result<Self, &str>{
        println!("lade {:?}", fname);

        let err_msg = "Error parsing file.";

        let f = File::open(fname).map_err(|_| (err_msg))?;
        let f = BufReader::new(f);

        // ** Actual reading **

        let mut liter = f.lines();

        // "Header" / comments
        // Ignore all comments / type codes (assume 'MatrixMarket matrix coordinate real general format')
        let (mut m, mut n, mut l): (usize, usize, usize) = (0,0,0);
        for line in liter.by_ref() {
            let line = String::from(line.map_err(|_| (err_msg))?.trim());
            if line.starts_with("%"){
                continue;
            } else {
                let mut hspl = line.split(' ');
                m = hspl.next().ok_or(err_msg)?.parse().map_err(|_| (err_msg))?;
                n = hspl.next().ok_or(err_msg)?.parse().map_err(|_| (err_msg))?;
                l = hspl.next().ok_or(err_msg)?.parse().map_err(|_| (err_msg))?;
                break;
            }            
        }

        let shape = (m,n);

        // println!("{} {} {}", m, n, l);

        // let mut values = vec![vec![0.;n];m];
        let mut data: Vec<(usize, usize, f64)> = Vec::with_capacity(l);

        // //let mut i = 0;
        for _ in 0..l {
            let line = String::from(liter.next().expect(err_msg).unwrap().trim());
            let mut split = line.split(' ');
            
            let i: usize = split.next().ok_or(err_msg)?.parse::<usize>().map_err(|_| (err_msg))? -1;  // Start counting with 0
            let j: usize = split.next().ok_or(err_msg)?.parse::<usize>().map_err(|_| (err_msg))? -1; 
            let v: f64 = split.next().ok_or(err_msg)?.parse().map_err(|_| (err_msg))?;
            
            data.push((i,j,v));
        }

        Ok(COO{data, shape})
    }


    // Print the matrix nicely
    pub fn print(&self) {
        println!("Sparse ({},{})-matrix in COO format with {} entries", self.shape.0, self.shape.1, self.data.len());
        for (i,j,x) in &self.data {
            println!("self[{}][{}] = {}", i,j,x);
        }
    }

    




}