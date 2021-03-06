//! A vector-like type that allows for aggregate operations similar to python's `pandas.Series`.
//!
// TODO: add info on performance.
//!
//! # Examples
//! You can create a new [`Series`] using [`new`]:
//!
//! [`Series`]: ./struct.Series.html
//! [`new`]: ./struct.Series.html#method.new
//! ```
//! use raccoon::{Series, DataType};
//!
//! // create a series with name "Name" containing integers (defaults to `i32`)
//! let series = Series::new("Name".to_owned(), DataType::Integer);
//! ```
//!
//! Alternatively, you can create a `Series` with data [using a vector]:
//!
//! [using a vector]: ./struct.Series.html#method.from_vector
//! ```
//! use raccoon::{Series, DataEntry};
//!
//! let v = vec![true, false, true];
//! let series = Series::from_vector("bools".to_owned(), v);
//!
//! // data type is infered from data passed to it
//! assert_eq!(series[1usize], DataEntry::Boolean(false));
//! ```
//!
//! You can also push new values onto the end of a `Series`:
//! ```
//! use raccoon::{Series, DataEntry, DataType};
//!
//! let mut series = Series::from(vec![0u32, 1u32, 2u32]);
//! assert_eq!(series.data_type(), &DataType::UInteger);
//!
//! let result = series.push(3u32);
//! assert!(result.is_ok());
//! assert_eq!(series[3usize], DataEntry::UInteger(3u32));
//!
//! let result = series.push(false);
//! assert!(result.is_err());
//!
//! // you can also push vectors
//! let _ = series.push_vec(vec![4u32, 5u32, 6u32]);
//! assert_eq!(series[5usize], DataEntry::UInteger(5u32));
//!
//! // or push `DataEntry`s
//! let _ = series.push_entry(DataEntry::UInteger(7u32));
//! let _ = series.push_entry_vec(vec![DataEntry::UInteger(8u32), DataEntry::UInteger(9u32)]);
//! assert_eq!(series[9usize], DataEntry::UInteger(9u32));
//! ```


use entry::{DataEntry, DataType};
use error::{RaccoonResult, RaccoonError};

use std::ops::Index;

/// A growable, named series. This tries to conform to the behaviour of python's `pandas.Series`.
///
/// # Examples
/// ```
/// use raccoon::{Series, DataType, DataEntry};
///
/// let mut series = Series::new("My Series".to_owned(), DataType::Double);
/// series.push(3.45f64);
/// series.push(67.8f64);
///
/// assert_eq!(series.len(), 2);
/// assert_eq!(series[0], DataEntry::Double(3.45));
///
/// series.push_vec(vec![2.0f64, 2.1, 2.2, 2.3]);
/// ```
///
/// In general, this can be seen as a special type of vector, allowing aggregate operations. However, one major
/// major difference, is the fact that a `Series` **cannot be indexed mutably**. Hence code such as the following will
/// cause a compilation error:
/// ```ignore
/// let mut series = Series::from(vec![1, 2, 3]);
/// series[0] = 5;          // compile time error
/// ```
/// The reason for prohibiting mutable indexing is to ensure data type integrity. Code such as the following would
/// otherwise run without problems:
/// ```ignore
/// // creating a series containing boolean values
/// let mut series = Series::from(vec![true, false, true]);
///
/// // setting the second value to an integer
/// series[1] = DataEntry::Integer(32);     // should NOT be allowed!!
/// ```
#[derive(Debug, Clone)]
pub struct Series {
    name: String,
    entries: Vec<DataEntry>,
    data_type: DataType,
}

impl Series {
    /// Constructs a new, empty `Series` with the specified name and data type.
    ///
    /// # Example
    /// ```
    /// use raccoon::{Series, DataType};
    ///
    /// let series = Series::new("My Series".to_owned(), DataType::Float);
    /// assert!(series.is_empty());
    /// assert_eq!("My Series", series.name());
    /// ```
    pub fn new(name: String, data_type: DataType) -> Series {
        Series {
            name: name,
            entries: Vec::new(),
            data_type: data_type
        }
    }

    /// Constructs a new, empty `Series` with the specified name, data type, and capacity.
    ///
    /// The series will be able to hold exactly `capacity` elements without reallocating. It is important to note that
    /// although the returned series will have the _capacity_ specified, the series will have zero length. See Rust's
    /// documentation of `std::vec::Vec<T>` for the difference between length and capacity.
    ///
    /// # Example
    /// ```
    /// use raccoon::{Series, DataType, DataEntry};
    ///
    /// let mut series = Series::with_capacity("series1".to_owned(), DataType::Integer, 10);
    ///
    /// // the series contain no items even though it has capacity for more
    /// assert!(series.is_empty());
    ///
    /// // these are all done without reallocation
    /// for i in 0i32..10i32 {
    ///     series.push(i);
    /// }
    ///
    /// // ... but this may make the series reallocate
    /// series.push(11);
    /// ```
    pub fn with_capacity(name: String, data_type: DataType, capacity: usize) -> Series {
        Series {
            name: name,
            entries: Vec::with_capacity(capacity),
            data_type: data_type
        }
    }

    /// Append a data entry to the series.
    ///
    /// As this uses type inference to add the data entry, ensure the append occured. `data` must match the internal
    /// type used by the series.
    ///
    /// # Example
    /// ```
    /// # use raccoon::{Series, DataType, DataEntry};
    /// // using `i32` to create the series
    /// let mut series = Series::from(vec![0, 1, 2, 3]);
    ///
    /// // ... hence the type is `DataType::Integer`
    /// assert_eq!(series.data_type(), &DataType::Integer);
    ///
    /// // works
    /// let result = series.push(4);
    /// assert!(result.is_ok());
    /// assert_eq!(series[4], DataEntry::Integer(4));
    ///
    /// // fails
    /// let result = series.push(5.0);      // f32
    /// assert!(result.is_err());
    /// ```
    pub fn push<T>(&mut self, data: T) -> RaccoonResult where T: Into<DataEntry> {
        let data_entry: DataEntry = data.into();
        self.push_entry(data_entry)
    }

    /// Extend the series by a data vector.
    ///
    /// As this uses type inference to add the data entry, ensure the append occured. `data` must match the internal
    /// type used by the series.
    /// # Example
    /// ```
    /// # use raccoon::{Series, DataType, DataEntry};
    /// // using `i32` to create the series
    /// let mut series = Series::from(vec![0, 1, 2, 3]);
    ///
    /// // ... hence the type is `DataType::Integer`
    /// assert_eq!(series.data_type(), &DataType::Integer);
    ///
    /// // works
    /// let result = series.push_vec(vec![4, 5, 6]);
    /// assert!(result.is_ok());
    /// assert_eq!(series[6], DataEntry::Integer(6));
    ///
    /// // fails
    /// let result = series.push_vec(vec![3.4, 5.6, 1.2]);      // f32
    /// assert!(result.is_err());
    /// ```
    pub fn push_vec<T>(&mut self, vector: Vec<T>) -> RaccoonResult where T: Into<DataEntry> {
        let entries: Vec<DataEntry> = vector.into_iter().map(|x| x.into()).collect();
        self.push_entry_vec(entries)
    }

    /// Append a `DataEntry` object to the series.
    ///
    /// # Example
    /// ```
    /// # use raccoon::{Series, DataType, DataEntry};
    /// // using `i32` to create the series
    /// let mut series = Series::from(vec![0, 1, 2, 3]);
    ///
    /// // ... hence the type is `DataType::Integer`
    /// assert_eq!(series.data_type(), &DataType::Integer);
    ///
    /// // works
    /// let result = series.push_entry(DataEntry::Integer(4));
    /// assert!(result.is_ok());
    /// assert_eq!(series[4], DataEntry::Integer(4));
    ///
    /// // fails
    /// let result = series.push(DataEntry::Float(5.0));
    /// assert!(result.is_err());
    /// ```
    pub fn push_entry(&mut self, data_entry: DataEntry) -> RaccoonResult {
        if !self.verify_type(data_entry.data_type()) {
            return Err(RaccoonError::InvalidType);
        }
        self.entries.push(data_entry);
        Ok(())
    }

    /// Append a `DataEntry` vector to the series.
    ///
    /// # Example
    /// ```
    /// # use raccoon::{Series, DataType, DataEntry};
    /// // using `i32` to create the series
    /// let mut series = Series::from(vec![0, 1, 2, 3]);
    ///
    /// // ... hence the type is `DataType::Integer`
    /// assert_eq!(series.data_type(), &DataType::Integer);
    ///
    /// // works
    /// let vector = vec![
    ///     DataEntry::Integer(4),
    ///     DataEntry::Integer(5),
    ///     DataEntry::Integer(6),
    /// ];
    /// let result = series.push_entry_vec(vector);
    /// assert!(result.is_ok());
    /// assert_eq!(series[6], DataEntry::Integer(6));
    ///
    /// // fails
    /// let vector = vec![
    ///     DataEntry::Float(3.4),
    ///     DataEntry::Float(5.6),
    ///     DataEntry::Float(1.2),
    /// ];
    /// let result = series.push_entry_vec(vector);
    /// assert!(result.is_err());
    /// ```
    pub fn push_entry_vec(&mut self, vector: Vec<DataEntry>) -> RaccoonResult {
        if vector.iter().any(|ref x| !self.verify_type(x.data_type())) {
            return Err(RaccoonError::InvalidType);
        }
        for item in vector {
            self.entries.push(item);
        }
        Ok(())
    }

    /// Pops an entry from the end of the series.
    ///
    /// # Example
    /// ```
    /// # use raccoon::{Series, DataType, DataEntry};
    /// let mut series = Series::new("series1".to_owned(), DataType::Boolean);
    /// series.push(true);
    ///
    /// assert_eq!(1, series.len());
    /// let result = series.pop_entry();
    /// assert_eq!(Some(DataEntry::Boolean(true)), result);
    ///
    /// assert!(series.is_empty());
    /// let result = series.pop_entry();
    /// assert_eq!(None, result);
    /// ```
    pub fn pop_entry(&mut self) -> Option<DataEntry> {
        self.entries.pop()
    }

    /// Returns the length of the series.
    ///
    /// # Example
    /// ```
    /// # use raccoon::Series;
    /// let series = Series::from(vec![1, 2, 3]);
    /// assert_eq!(3, series.len())
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Converts the series into another data type.
    ///
    /// Note that some data types cannot be converted into one another as the conversion makes no sense. This results in
    /// `DataType::NA` entries. The conversion from numerical types into boolean values is performed by checking
    /// equality with 0.
    ///
    /// # Conversions that result in `DataType::NA`
    /// - `DataType::Text` into another type that cannot be parsed into another type using `String::from()`. For example
    ///   the conversion shown in the third example of this docstring.
    /// - `DataType::Character` into `DataType::Boolean`.
    /// - Anything except `DataType::Text` into `DataType::Character`. This can be somewhat circumvented by converting
    ///   to `DataType::Text` and then into `DataType::Character`.
    /// - Any signed numerical type into an unsigned one.
    /// - `DataType::Long` into `DataType::Integer`.
    /// - `DataType::ULong` into `DataType::UInteger`.
    ///
    /// # Examples
    /// A working conversion:
    /// ```
    /// # use raccoon::{Series, DataType};
    /// let mut series = Series::from(vec![true, true, false, true]);
    /// series.convert_to(&DataType::Integer);
    /// assert_eq!(series, vec![1, 1, 0, 1]);
    /// ```
    ///
    /// A working yet lossy conversion:
    /// ```
    /// # use raccoon::{Series, DataType};
    /// // build double precision floating point series
    /// let mut series = Series::from(vec![123.456f64, 456.789f64]);
    /// assert_eq!(series.data_type(), &DataType::Double);
    ///
    /// // convert to single precision floating point
    /// series.convert_to(&DataType::Float);
    /// assert_eq!(series.data_type(), &DataType::Float);
    /// assert_eq!(series, vec![123.456f32, 456.789f32]);
    /// ```
    ///
    /// A conversion that makes no sense:
    /// ```
    /// # use raccoon::{Series, DataType, DataEntry};
    /// let mut series = Series::from(vec!["some", "random", "words"]);
    /// series.convert_to(&DataType::Character);
    /// assert_eq!(series, vec![DataEntry::NA, DataEntry::NA, DataEntry::NA]);
    /// ```
    pub fn convert_to(&mut self, data_type: &DataType) {
        let mut converted_entries: Vec<DataEntry> = Vec::new();
        for entry in &self.entries {
            converted_entries.push(entry.convert_to(data_type));
        }
        self.entries = converted_entries;
        self.data_type = data_type.clone();
    }

    /// Getter for the series' data type.
    ///
    /// # Example
    /// ```
    /// # use raccoon::{Series, DataType};
    /// let series = Series::new("my series".to_owned(), DataType::ULong);
    /// assert_eq!(series.data_type(), &DataType::ULong);
    /// ```
    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    /// Verifies the validity of the datatype. This checks if a given data type is conform to this series.
    ///
    /// In other words, this will return `true` if `data_type` is `DataType::NA` or equal to the data type of the
    /// series.
    fn verify_type(&self, data_type: DataType) -> bool {
        if data_type != self.data_type && data_type != DataType::NA {
            false
        } else {
            true
        }
    }

    /// Builds a `Series` from a vector of items and gives the series a name.
    ///
    /// # Example
    /// ```
    /// # use raccoon::Series;
    /// let series = Series::from_vector("my series".to_owned(), vec![1, 2, 3]);
    /// assert_eq!(series.name(), "my series");
    /// assert_eq!(series, vec![1, 2, 3]);
    /// ```
    pub fn from_vector<T>(name: String, vector: Vec<T>) -> Series where T: Into<DataEntry> {
        let entries: Vec<DataEntry> = vector.into_iter().map(|x| x.into()).collect();
        let mut data_type = DataType::NA;
        if !entries.is_empty() {
            data_type = entries[0].data_type().clone();
        }
        Series {
            name: name,
            entries: entries,
            data_type: data_type,
        }
    }

    /// Getter for the series' name.
    ///
    /// # Example
    /// ```
    /// # use raccoon::{Series, DataType};
    /// let series = Series::new("custom name".to_owned(), DataType::Character);
    /// assert_eq!(series.name(), "custom name");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Setter for the series' name.
    ///
    /// # Example
    /// ```
    /// # use raccoon::Series;
    /// let mut series = Series::from(vec!['a', 'b', 'c']);
    /// assert_eq!(series.name(), "Series1");
    ///
    /// // change name
    /// series.set_name("custom name".to_owned());
    /// assert_eq!(series.name(), "custom name");
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Checks if the series is empty.
    ///
    /// # Example
    /// ```
    /// # use raccoon::{Series, DataType};
    /// let mut series = Series::new("City".to_owned(), DataType::Text);
    /// assert!(series.is_empty());
    ///
    /// series.push("Zürich");
    /// assert!(!series.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<T> From<Vec<T>> for Series where T: Into<DataEntry> {
    fn from(vector: Vec<T>) -> Self {
        Series::from_vector("Series1".to_owned(), vector)
    }
}

impl Index<usize> for Series {
    type Output = DataEntry;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.entries[idx]
    }
}

impl PartialEq for Series {
    fn eq(&self, other: &Series) -> bool {
        if self.name == other.name && self.data_type == other.data_type && self.entries.len() == other.entries.len() {
            if self.entries.iter().zip(other.entries.iter()).all(|(ref x1, ref x2)| { x1 == x2 }) {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl<T> PartialEq<Vec<T>> for Series where DataEntry: From<T>, T: Clone {
    fn eq(&self, other: &Vec<T>) -> bool {
        if self.entries.iter().zip(other.iter()).all(|(x1, x2)| { *x1 == DataEntry::from(x2.clone()) }) {
            true
        } else {
            false
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_series() {
        let name = String::from("Dog breeds");
        let mut series = Series::new(name, DataType::Text);
        let result = series.push_entry(DataEntry::Text("Labrador".to_owned()));
        assert!(result.is_ok());
        let result = series.push_entry(DataEntry::NA);
        assert!(result.is_ok());
        let result = series.push_entry(DataEntry::Text("Golden retriever".to_owned()));
        assert!(result.is_ok());
        let result = series.push_entry(DataEntry::Integer(25));
        assert!(result.is_err());
        assert_eq!("Dog breeds", series.name());
        assert_eq!(3, series.len());
    }

    #[test]
    fn convert_series() {
        let name = String::from("Numbers");
        let mut series = Series::new(name, DataType::Text);
        let items = vec![
            DataEntry::Text("1".to_owned()),
            DataEntry::Text("2".to_owned()),
            DataEntry::Text("3".to_owned()),
            DataEntry::Text("".to_owned()),
            DataEntry::Text("4".to_owned())
        ];
        let result = series.push_entry_vec(items);
        assert!(result.is_ok());
        series.convert_to(&DataType::UInteger);
        assert_eq!("Numbers", series.name());
        assert_eq!(&DataType::UInteger, series.data_type());
        assert_eq!(DataEntry::UInteger(1u32), series[0usize]);
        assert_eq!(DataEntry::UInteger(2u32), series[1usize]);
        assert_eq!(DataEntry::UInteger(3u32), series[2usize]);
        assert_eq!(DataEntry::NA, series[3usize]);
        assert_eq!(DataEntry::UInteger(4u32), series[4usize]);
    }

    #[test]
    fn push_raw_entries() {
        let mut series = Series::new("name".to_owned(), DataType::Integer);
        let result = series.push_vec(vec![0, 1, 2]);
        assert!(result.is_ok());
        assert_eq!(DataEntry::Integer(0i32), series[0usize]);
        assert_eq!(DataEntry::Integer(1i32), series[1usize]);
        assert_eq!(DataEntry::Integer(2i32), series[2usize]);
        let result = series.push(3);
        assert!(result.is_ok());
        assert_eq!(DataEntry::Integer(3i32), series[3usize]);
        let result = series.push(true);
        assert!(result.is_err());
    }

    #[test]
    fn pop_items() {
        let mut series = Series::from(vec![1, 2, 3, 4]);
        assert_eq!(Some(DataEntry::Integer(4i32)), series.pop_entry());
        assert_eq!(Some(DataEntry::Integer(3i32)), series.pop_entry());
        assert_eq!(Some(DataEntry::Integer(2i32)), series.pop_entry());
        assert_eq!(Some(DataEntry::Integer(1i32)), series.pop_entry());
        assert_eq!(None, series.pop_entry());
        assert_eq!(None, series.pop_entry());
        assert!(series.is_empty());
    }

    #[test]
    fn construction_from_vector() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7];
        let series = Series::from_vector("Some series".to_owned(), vec);
        assert_eq!("Some series", series.name());
        assert_eq!(&DataType::Integer, series.data_type());
        assert_eq!(DataEntry::Integer(1), series[0usize]);
        assert_eq!(DataEntry::Integer(2), series[1usize]);
        assert_eq!(DataEntry::Integer(3), series[2usize]);
        assert_eq!(DataEntry::Integer(4), series[3usize]);
        assert_eq!(DataEntry::Integer(5), series[4usize]);
        assert_eq!(DataEntry::Integer(6), series[5usize]);
        assert_eq!(DataEntry::Integer(7), series[6usize]);

        let vec: Vec<u64> = Vec::new();
        let series = Series::from_vector("empty".to_owned(), vec);
        assert_eq!("empty", series.name());
        assert_eq!(&DataType::NA, series.data_type());
    }

    #[test]
    #[should_panic(expected="index out of bounds: the len is 0 but the index is 0")]
    fn empty_indexing() {
        let vec: Vec<u64> = Vec::new();
        let series = Series::from(vec);
        let _ = &series[0_usize];
    }

    #[test]
    fn construction_from_trait() {
        let vec = vec![true, false, false, true, false, true, true];
        let mut series = Series::from(vec);
        assert_eq!("Series1", series.name());
        series.set_name("My new name".to_owned());
        assert_eq!("My new name", series.name());
        assert_eq!(&DataType::Boolean, series.data_type());
        assert_eq!(DataEntry::Boolean(true), series[0usize]);
        assert_eq!(DataEntry::Boolean(false), series[1usize]);
        assert_eq!(DataEntry::Boolean(false), series[2usize]);
        assert_eq!(DataEntry::Boolean(true), series[3usize]);
        assert_eq!(DataEntry::Boolean(false), series[4usize]);
        assert_eq!(DataEntry::Boolean(true), series[5usize]);
        assert_eq!(DataEntry::Boolean(true), series[6usize]);
        assert_eq!(7, series.len());
    }

    #[test]
    fn comparisions() {
        let series1 = Series::from(vec![1, 2, 3, 4]);
        assert_eq!(series1, vec![1, 2, 3, 4]);
        let series2 = Series::from(vec![1, 2, 3, 4]);
        assert_eq!(series1, series2);
        let mut series3 = Series::from(vec![1, 2, 3, 4]);
        series3.set_name("some random name".to_owned());
        assert_ne!(series1, series3);
        let series4 = Series::from(vec![1.0, 2.0, 3.0, 4.0]);
        assert_ne!(series1, series4);
    }
}
