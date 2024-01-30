use csv::ReaderBuilder;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug)]
pub(crate) struct Nation {
    pub n_nationkey: u64,
    pub n_name: String,
    pub n_regionkey: u64,
    pub n_comment: String,
}

#[derive(Debug)]
pub(crate) struct Region {
    pub r_regionkey: u64,
    pub r_name: String,
    pub r_comment: String,
}

#[derive(Debug)]
pub(crate) struct Part {
    pub p_partkey: u64,
    pub p_name: String,
    pub p_mfgr: String,
    pub p_brand: String,
    pub p_type: String,
    pub p_size: u64,
    pub p_container: String,
    pub p_retailprice: f64,
    pub p_comment: String,
}

#[derive(Debug)]
pub(crate) struct Customer {
    pub c_custkey: u64,
    pub c_name: String,
    pub c_address: String,
    pub c_nationkey: String,
    pub c_phone: String,
    pub c_acctbal: f64,
    pub c_mktsegment: String,
    pub c_comment: String,
}

#[derive(Debug)]
pub(crate) struct Lineitem {
    pub l_orderkey: u64,
    pub l_partkey: u64,
    pub l_suppkey: u64,
    pub l_linenumber: u64,
    pub l_quantity: u64,
    pub l_extendedprice: f64,
    pub l_discount: f64,
    pub l_tax: f64,
    pub l_returnflag: String,
    pub l_linestatus: String,
    pub l_shipdate: String,
    pub l_commitdate: String,
    pub l_receiptdate: String,
    pub l_shipinstruct: String,
    pub l_shipmode: String,
    pub l_comment: String,
}

#[derive(Debug)]
pub(crate) struct Orders {
    pub o_orderkey: u64,
    pub o_custkey: u64,
    pub o_orderstatus: String,
    pub o_totoalprice: f64,
    pub o_orderdate: String,
    pub o_orderpriority: String,
    pub o_clerk: String,
    pub o_shippriority: u64,
    pub o_comment: String,
}

#[derive(Debug)]
pub(crate) struct Partsupp {
    pub ps_partkey: u64,
    pub ps_suppkey: u64,
    pub ps_availqty: u64,
    pub ps_supplycost: f64,
    pub ps_comment: String,
}

#[derive(Debug)]
pub(crate) struct Supplier {
    pub s_suppkey: u64,
    pub s_name: String,
    pub s_address: String,
    pub s_nationkey: u64,
    pub s_phone: String,
    pub s_acctbal: f64,
    pub s_comment: String,
}

// Function to read records from a .tbl file
pub(crate) fn nation_read_records_from_file(file_path: &str) -> Result<Vec<Nation>, io::Error> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let mut records: Vec<Nation> = Vec::new();

    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let fields: Vec<&str> = line.split('|').collect();

        let record = Nation {
            n_nationkey: fields[0].parse().unwrap(),
            n_name: fields[1].to_string(),
            n_regionkey: fields[2].parse().unwrap(),
            n_comment: fields[3].to_string(),
        };

        records.push(record);
    }

    Ok(records)
}

pub(crate) fn part_read_records_from_file(file_path: &str) -> Result<Vec<Part>, io::Error> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let mut records: Vec<Part> = Vec::new();

    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let fields: Vec<&str> = line.split('|').collect();

        let record = Part {
            p_partkey: fields[0].parse().unwrap(),
            p_name: fields[1].to_string(),
            p_mfgr: fields[2].to_string(),
            p_brand: fields[3].to_string(),
            p_type: fields[4].to_string(),
            p_size: fields[5].parse().unwrap(),
            p_container: fields[6].to_string(),
            p_retailprice: fields[0].parse().unwrap(),
            p_comment: fields[8].to_string(),
        };

        records.push(record);
    }

    Ok(records)
}

pub(crate) fn customer_read_records_from_file(file_path: &str) -> Result<Vec<Customer>, io::Error> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let mut records: Vec<Customer> = Vec::new();

    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let fields: Vec<&str> = line.split('|').collect();

        let record = Customer {
            c_custkey: fields[0].parse().unwrap(),
            c_name: fields[1].to_string(),
            c_address: fields[2].to_string(),
            c_nationkey: fields[3].to_string(),
            c_phone: fields[4].to_string(),
            c_acctbal: fields[5].parse().unwrap(),
            c_mktsegment: fields[6].to_string(),
            c_comment: fields[7].to_string(),
        };

        records.push(record);
    }

    Ok(records)
}

pub(crate) fn lineitem_read_records_from_file(file_path: &str) -> Result<Vec<Lineitem>, io::Error> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let mut records: Vec<Lineitem> = Vec::new();

    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let fields: Vec<&str> = line.split('|').collect();

        let record = Lineitem {
            l_orderkey: fields[0].parse().unwrap(),
            l_partkey: fields[1].parse().unwrap(),
            l_suppkey: fields[2].parse().unwrap(),
            l_linenumber: fields[3].parse().unwrap(),
            l_quantity: fields[4].parse().unwrap(),
            l_extendedprice: fields[5].parse().unwrap(),
            l_discount: fields[6].parse().unwrap(),
            l_tax: fields[7].parse().unwrap(),
            l_returnflag: fields[8].to_string(),
            l_linestatus: fields[9].to_string(),
            l_shipdate: fields[10].to_string(),
            l_commitdate: fields[11].to_string(),
            l_receiptdate: fields[12].to_string(),
            l_shipinstruct: fields[13].to_string(),
            l_shipmode: fields[14].to_string(),
            l_comment: fields[15].to_string(),
        };

        records.push(record);
    }

    Ok(records)
}

pub(crate) fn orders_read_records_from_file(file_path: &str) -> Result<Vec<Orders>, io::Error> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let mut records: Vec<Orders> = Vec::new();

    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let fields: Vec<&str> = line.split('|').collect();

        let record = Orders {
            o_orderkey: fields[0].parse().unwrap(),
            o_custkey: fields[1].parse().unwrap(),
            o_orderstatus: fields[2].to_string(),
            o_totoalprice: fields[3].parse().unwrap(),
            o_orderdate: fields[4].to_string(),
            o_orderpriority: fields[5].parse().unwrap(),
            o_clerk: fields[6].to_string(),
            o_shippriority: fields[7].parse().unwrap(),

            o_comment: fields[8].to_string(),
        };

        records.push(record);
    }

    Ok(records)
}

pub(crate) fn partsupp_read_records_from_file(file_path: &str) -> Result<Vec<Partsupp>, io::Error> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let mut records: Vec<Partsupp> = Vec::new();

    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let fields: Vec<&str> = line.split('|').collect();

        let record = Partsupp {
            ps_partkey: fields[0].parse().unwrap(),
            ps_suppkey: fields[1].parse().unwrap(),
            ps_availqty: fields[2].parse().unwrap(),
            ps_supplycost: fields[3].parse().unwrap(),
            ps_comment: fields[4].to_string(),
        };

        records.push(record);
    }

    Ok(records)
}

pub(crate) fn supplier_read_records_from_file(file_path: &str) -> Result<Vec<Supplier>, io::Error> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let mut records: Vec<Supplier> = Vec::new();

    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let fields: Vec<&str> = line.split('|').collect();

        let record = Supplier {
            s_suppkey: fields[0].parse().unwrap(),
            s_name: fields[1].to_string(),
            s_address: fields[2].to_string(),
            s_nationkey: fields[3].parse().unwrap(),
            s_phone: fields[4].to_string(),
            s_acctbal: fields[5].parse().unwrap(),
            s_comment: fields[6].to_string(),
        };

        records.push(record);
    }

    Ok(records)
}

// Function to read records from a .csv file
pub(crate) fn region_read_records_from_csv(file_path: &str) -> Result<Vec<Region>, csv::Error> {
    let file = File::open(file_path)?;
    // let mut reader = ReaderBuilder::new().delimiter(b'|').from_reader(file);
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .from_reader(file);

    let mut records: Vec<Region> = Vec::new();

    for result in reader.records() {
        let record = result?;
        let r_regionkey: u64 = record[0].trim().parse().unwrap();
        let r_name = record[1].trim().to_string();
        let r_comment = record[2].trim().to_string();

        records.push(Region {
            r_regionkey,
            r_name,
            r_comment,
        });
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        fn string_to_u64(s: &str) -> u64 {
            let mut result = 0;

            for (i, c) in s.chars().enumerate() {
                result += (i as u64 + 1) * (c as u64);
            }

            result
        }
        fn scale_by_1000(x: f64) -> u64 {
            (1000.0 * x) as u64
        }
        // Call the function with different file paths

        if let Ok(records) =
            customer_read_records_from_file("/Users/binbingu/halo2-TPCH/src/data/customer.tbl")
        {
            println!("{:?}", string_to_u64(&records[0].c_mktsegment));
        } else {
            println!("Failed to read records from file1");
        }

        if let Ok(records) =
            orders_read_records_from_file("/Users/binbingu/halo2-TPCH/src/data/orders.tbl")
        {
            println!("{:?}", string_to_u64(&records[0].o_orderdate));
        } else {
            println!("Failed to read records from file1");
        }

        if let Ok(records) =
            lineitem_read_records_from_file("/Users/binbingu/halo2-TPCH/src/data/lineitem.tbl")
        {
            println!("{:?}", string_to_u64(&records[0].l_shipdate));
            // println!("{:?}", string_to_u64(&records[1].l_shipdate));
            // println!("{:?}", string_to_u64(&records[2].l_shipdate));
        } else {
            println!("Failed to read records from file1");
        }

        // if let Ok(records) =
        //     region_read_records_from_csv("/Users/binbingu/halo2-TPCH/src/data/region.cvs")
        // {
        //     println!("{:?}", string_to_u64(&records[0].r_name));
        // } else {
        //     println!("Failed to read records from file2");
        // }
    }
}
