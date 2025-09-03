use primitive_types::U256;

use crate::UTnfa;

/// Set of utf8-characters
pub struct Utf8Charset {
    ranges: Vec<(char, char)>,
    invert: bool,
}

/// Set of single-byte characters, including `'\u{80}'..'\u{ff}'`.
/// Multi-byte character can be represented as `Utf8Charset`
#[derive(Clone, Copy)]
pub struct Charset {
    c: U256,
}

impl Utf8Charset {
    /// Creates an empty utf-8 charset
    pub fn empty() -> Self {
        Self {
            ranges: Vec::new(),
            invert: false,
        }
    }

    /// Inverts the charset, i.e. applies `'^'` operator
    pub fn invert(&mut self, invert: bool) {
        self.invert = invert;
    }

    /// Adds char `c` to `self`
    pub fn add_char(&mut self, c: char) {
        self.add_range((c, c));
    }

    /// Adds all characters in range `range.0..=range.1` to `self`
    pub fn add_range(&mut self, range: (char, char)) {
        self.ranges.push(range);
    }
}

impl Charset {
    /// Creates an empty charset
    pub fn empty() -> Self {
        Self { c: U256::zero() }
    }

    /// Creates a charset, that contains character `c`
    pub fn from_char(c: u8) -> Self {
        Self::from_range((c, c))
    }

    /// Creates a charset, that contains all characters within `r.0..=r.1`
    pub fn from_range(r: (u8, u8)) -> Self {
        let mut s = Self::empty();
        for c in r.0..=r.1 {
            s.c |= U256::one() << c;
        }
        s
    }

    /// Returns iterator over all chars, contained within charset
    pub fn iter(&self) -> impl Iterator<Item = u8> {
        CharsetIter { c: *self, i: 0 }
    }

    /// Returns `true` if `self` contains char `c`
    pub fn contains(&self, c: u8) -> bool {
        (self.c & (U256::one() << c)) != U256::zero()
    }
}

impl std::ops::BitOrAssign for Charset {
    fn bitor_assign(&mut self, rhs: Self) {
        self.c |= rhs.c;
    }
}

struct CharsetIter {
    c: Charset,
    i: usize,
}

impl Iterator for CharsetIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.i += 1;
            match (self.i - 1).try_into() {
                Err(_) => return None,
                Ok(c) if self.c.contains(c) => return Some(c),
                _ => continue,
            }
        }
    }
}

/// Creates charset
#[macro_export]
macro_rules! charset {
    ($($a:literal $(- $b:literal)?)*) => {
        charset!(@impl false, $($a)*, $($($a-$b)?)*)
    };
    (^ $($a:literal $(- $b:literal)?)*) => {
        charset!(@impl true, $($a)*, $($($a-$b)?)*)
    };
    (@impl $inv:ident, $($a:literal)*, $($b:literal-$c:literal)*) => {
        {
            let mut c = Utf8Charset::empty();
            $(c.add_char($a);)*
            $(c.add_range(($b, $c));)*
            c.invert($inv);
            Into::<UTnfa>::into(c)
        }
    }
}

// Following code implements Into<UTnfa> for Utf8Charset

const UTF8_RANGES: [(char, char); 4] = [
    ('\u{000000}', '\u{00007f}'),
    ('\u{000080}', '\u{0007ff}'),
    ('\u{000800}', '\u{00ffff}'),
    ('\u{010000}', '\u{10ffff}'),
];

/// Calculates intersection between 2 character ranges
/// If it the result is empty `None` is returned
fn intersect_ranges(a: (char, char), b: (char, char)) -> Option<(char, char)> {
    let (a, b) = (std::cmp::max(a.0, b.0), std::cmp::min(a.1, b.1));
    if a > b { None } else { Some((a, b)) }
}

/// Subtracts ranges `sub` from original range `a` and returns resulting list of ranges
/// Here we are using a simple dp to iteratively calculate result
fn subtract_ranges(a: &[(char, char)], sub: &[(char, char)]) -> Box<[(char, char)]> {
    let mut dp = [Vec::from_iter(a.iter().map(|a| *a)), Vec::new()];
    for (i, s) in sub.iter().enumerate().map(|(i, s)| (i % 2, s)) {
        dp[i ^ 1].clear();
        for j in 0..dp[i].len() {
            let old = dp[i][j];
            match intersect_ranges(old, *s) {
                None => dp[i ^ 1].push(old),
                Some(s) => {
                    if old.0 < s.0 {
                        // SAFETY: old.0 is a valid character and old.0 < s.0
                        dp[i ^ 1].push((old.0, unsafe { char::from_u32_unchecked(s.0 as u32 - 1) }))
                    }
                    if old.1 > s.1 {
                        // SAFETY: old.1 is a valid character and old.1 > s.1
                        dp[i ^ 1].push((unsafe { char::from_u32_unchecked(s.1 as u32 + 1) }, old.1))
                    }
                }
            }
        }
    }

    // Some fighting with borrow-checker happened here
    // SAFETY: dp.len() is always 2 and sub.len() % 2 is always less than 2
    unsafe {
        dp.into_iter()
            .nth(sub.len() % 2)
            .unwrap_unchecked()
            .into_boxed_slice()
    }
}

/// Creates UTnfa from character range
/// Algorithm:
/// 1. Ranges are splitted into smaller ranges, s.t. utf-8 representations all
/// characters in the same range have the same byte length
/// 2. For each range, a UTnfa is created (by concatenating UTnfa for Charsets for each byte)
/// 3. Theese UTnfa's are united
fn multibyte_range(a: char, b: char) -> UTnfa {
    let r = [
        intersect_ranges((a, b), UTF8_RANGES[0]),
        intersect_ranges((a, b), UTF8_RANGES[1]),
        intersect_ranges((a, b), UTF8_RANGES[2]),
        intersect_ranges((a, b), UTF8_RANGES[3]),
    ];

    let mut res = UTnfa::empty();
    for (count, r) in r.iter().enumerate().map(|(i, r)| (i + 1, r)) {
        match r {
            None => continue,
            Some((a, b)) => {
                let mut g = ([0; 4], [0, 4]);
                let mut u = UTnfa::empty();
                a.encode_utf8(&mut g.0);
                b.encode_utf8(&mut g.1);
                for i in 0..count {
                    u.concat(&UTnfa::charset(Charset::from_range((g.0[i], g.1[i]))));
                }
                res.union(&u);
            }
        }
    }

    UTnfa::empty()
}

impl Into<UTnfa> for Utf8Charset {
    fn into(self) -> UTnfa {
        let mut ranges = self.ranges.into_boxed_slice();
        if self.invert {
            ranges = subtract_ranges(&UTF8_RANGES, &ranges)
        }
        let mut res = UTnfa::empty();
        for range in ranges {
            res.union(&multibyte_range(range.0, range.1));
        }
        res
    }
}
