macro_rules! define_permissions {
  ($name:ident {
      $( $flag:ident = $val:expr ),* $(,)?
  }) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct $name(u64);

    impl $name {
      pub const EMPTY: Self = Self(0);
      pub const ALL: Self = Self(!0);

      $(pub const $flag: Self = Self($val);)*

      pub const fn bits(self) -> u64 {
        self.0
      }
      pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
      }
      pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
      }
      pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
      }
      pub const fn from_bits(bits: u64) -> Self {
          Self(bits)
      }
    }

    impl std::ops::BitOr for $name {
      type Output = Self;
      fn bitor(self, r: Self) -> Self {
        Self(self.0 | r.0)
      }
    }

    impl std::ops::BitAnd for $name {
      type Output = Self;
      fn bitand(self, r: Self) -> Self {
        Self(self.0 & r.0)
      }
    }

    impl std::ops::Not for $name {
      type Output = Self;
      fn not(self) -> Self {
        Self(!self.0)
      }
    }
  };
}
