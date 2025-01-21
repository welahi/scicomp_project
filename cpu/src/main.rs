use std::path::Path;

use fakscpu::cpu::{COO, CSR, Dense};

fn main() {
    println!("hi");

    let fname = Path::new("../matrix_instances/a001.mtx");
    let coo = COO::read_mtx(fname, false).expect(":(");
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

    // let a002 = COO::read_mtx("matrix_instances/generated/case_0000_A.mtx").expect(":(");
    let mut a002 = COO::read_mtx(Path::new("../matrix_instances/generated/case_0000_A.mtx"), true).expect(":(");
    // a002.to_dense().print();

    // a002.data.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    println!("{:?}", a002.data);


}
