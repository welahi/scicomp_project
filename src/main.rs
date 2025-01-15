

use faksgpu::cpu::{COO, CSR, Dense};

fn main() {
    println!("hi");

    let fname = "data/a01.mm";
    let fname = "data/a01.mm";
    let coo = COO::read_mtx(fname).expect(":(");
    coo.print();
    coo.to_dense().print();

    let csr = CSR::from_coo(coo);
    csr.print();

    let res = csr.product(&csr);
    res.print();


    let csr_d = csr.to_dense();
    csr_d.print();

    // for row in res {
    //     for x in row {
    //         print!("{}\t",x);
    //     }
    //     println!();
    // }

    // let d = Dense::new_zeros((3,3));
    // d.print();


}
