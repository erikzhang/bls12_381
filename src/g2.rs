//! This module provides an implementation of the $\mathbb{G}_2$ group of BLS12-381.

use crate::fp::Fp;
use crate::fp2::Fp2;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

/// This is an element of $\mathbb{G}_2$ represented in the affine coordinate space.
/// It is ideal to keep elements in this representation to reduce memory usage and
/// improve performance through the use of mixed curve model arithmetic.
///
/// Values of `G2Affine` are guaranteed to be in the $q$-order subgroup unless an
/// "unchecked" API was misused.
#[derive(Copy, Clone, Debug)]
pub struct G2Affine {
    x: Fp2,
    y: Fp2,
    infinity: Choice,
}

impl<'a> From<&'a G2Projective> for G2Affine {
    fn from(p: &'a G2Projective) -> G2Affine {
        let zinv = p.z.invert().unwrap_or(Fp2::zero());
        let zinv2 = zinv.square();
        let x = p.x * zinv2;
        let zinv3 = zinv2 * zinv;
        let y = p.y * zinv3;

        let tmp = G2Affine {
            x,
            y,
            infinity: Choice::from(0u8),
        };

        G2Affine::conditional_select(&tmp, &G2Affine::identity(), zinv.is_zero())
    }
}

impl From<G2Projective> for G2Affine {
    fn from(p: G2Projective) -> G2Affine {
        G2Affine::from(&p)
    }
}

impl ConstantTimeEq for G2Affine {
    fn ct_eq(&self, other: &Self) -> Choice {
        // The only cases in which two points are equal are
        // 1. infinity is set on both
        // 2. infinity is not set on both, and their coordinates are equal

        (self.infinity & other.infinity)
            | ((!self.infinity)
                & (!other.infinity)
                & self.x.ct_eq(&other.x)
                & self.y.ct_eq(&other.y))
    }
}

impl ConditionallySelectable for G2Affine {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        G2Affine {
            x: Fp2::conditional_select(&a.x, &b.x, choice),
            y: Fp2::conditional_select(&a.y, &b.y, choice),
            infinity: Choice::conditional_select(&a.infinity, &b.infinity, choice),
        }
    }
}

impl Eq for G2Affine {}
impl PartialEq for G2Affine {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.ct_eq(other))
    }
}

const B: Fp2 = Fp2 {
    c0: Fp::from_raw_unchecked([
        0xaa270000000cfff3,
        0x53cc0032fc34000a,
        0x478fe97a6b0a807f,
        0xb1d37ebee6ba24d7,
        0x8ec9733bbf78ab2f,
        0x9d645513d83de7e,
    ]),
    c1: Fp::from_raw_unchecked([
        0xaa270000000cfff3,
        0x53cc0032fc34000a,
        0x478fe97a6b0a807f,
        0xb1d37ebee6ba24d7,
        0x8ec9733bbf78ab2f,
        0x9d645513d83de7e,
    ]),
};

impl G2Affine {
    /// Returns the identity of the group: the point at infinity.
    pub fn identity() -> G2Affine {
        G2Affine {
            x: Fp2::zero(),
            y: Fp2::one(),
            infinity: Choice::from(1u8),
        }
    }

    /// Returns a fixed generator of the group. See [`notes::design`](notes/design/index.html#fixed-generators)
    /// for how this generator is chosen.
    pub fn generator() -> G2Affine {
        G2Affine {
            x: Fp2 {
                c0: Fp::from_raw_unchecked([
                    0xf5f28fa202940a10,
                    0xb3f5fb2687b4961a,
                    0xa1a893b53e2ae580,
                    0x9894999d1a3caee9,
                    0x6f67b7631863366b,
                    0x58191924350bcd7,
                ]),
                c1: Fp::from_raw_unchecked([
                    0xa5a9c0759e23f606,
                    0xaaa0c59dbccd60c3,
                    0x3bb17e18e2867806,
                    0x1b1ab6cc8541b367,
                    0xc2b6ed0ef2158547,
                    0x11922a097360edf3,
                ]),
            },
            y: Fp2 {
                c0: Fp::from_raw_unchecked([
                    0x4c730af860494c4a,
                    0x597cfa1f5e369c5a,
                    0xe7e6856caa0a635a,
                    0xbbefb5e96e0d495f,
                    0x7d3a975f0ef25a2,
                    0x83fd8e7e80dae5,
                ]),
                c1: Fp::from_raw_unchecked([
                    0xadc0fc92df64b05d,
                    0x18aa270a2b1461dc,
                    0x86adac6a3be4eba0,
                    0x79495c4ec93da33a,
                    0xe7175850a43ccaed,
                    0xb2bc2a163de1bf2,
                ]),
            },
            infinity: Choice::from(0u8),
        }
    }

    /// Returns true if this element is the identity (the point at infinity).
    #[inline]
    pub fn is_identity(&self) -> Choice {
        self.infinity
    }

    /// Returns true if this point is on the curve. This should always return
    /// true unless an "unchecked" API was used.
    pub fn is_on_curve(&self) -> Choice {
        // y^2 - x^3 ?= 4(u + 1)
        (self.y.square() - (self.x.square() * self.x)).ct_eq(&B) | self.infinity
    }
}

/// This is an element of $\mathbb{G}_2$ represented in the projective coordinate space.
#[derive(Copy, Clone, Debug)]
pub struct G2Projective {
    x: Fp2,
    y: Fp2,
    z: Fp2,
}

impl<'a> From<&'a G2Affine> for G2Projective {
    fn from(p: &'a G2Affine) -> G2Projective {
        G2Projective {
            x: p.x,
            y: p.y,
            z: Fp2::conditional_select(&Fp2::one(), &Fp2::zero(), p.infinity),
        }
    }
}

impl From<G2Affine> for G2Projective {
    fn from(p: G2Affine) -> G2Projective {
        G2Projective::from(&p)
    }
}

impl ConstantTimeEq for G2Projective {
    fn ct_eq(&self, other: &Self) -> Choice {
        // Is (xz^2, yz^3, z) equal to (x'z'^2, yz'^3, z') when converted to affine?

        let z = other.z.square();
        let x1 = self.x * z;
        let z = z * other.z;
        let y1 = self.y * z;
        let z = self.z.square();
        let x2 = other.x * z;
        let z = z * self.z;
        let y2 = other.y * z;

        let self_is_zero = self.z.is_zero();
        let other_is_zero = other.z.is_zero();

        (self_is_zero & other_is_zero) // Both point at infinity
            | ((!self_is_zero) & (!other_is_zero) & x1.ct_eq(&x2) & y1.ct_eq(&y2)) // Neither point at infinity, coordinates are the same
    }
}

impl ConditionallySelectable for G2Projective {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        G2Projective {
            x: Fp2::conditional_select(&a.x, &b.x, choice),
            y: Fp2::conditional_select(&a.y, &b.y, choice),
            z: Fp2::conditional_select(&a.z, &b.z, choice),
        }
    }
}

impl Eq for G2Projective {}
impl PartialEq for G2Projective {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.ct_eq(other))
    }
}

impl G2Projective {
    /// Returns the identity of the group: the point at infinity.
    pub fn identity() -> G2Projective {
        G2Projective {
            x: Fp2::zero(),
            y: Fp2::one(),
            z: Fp2::zero(),
        }
    }

    /// Returns a fixed generator of the group. See [`notes::design`](notes/design/index.html#fixed-generators)
    /// for how this generator is chosen.
    pub fn generator() -> G2Projective {
        G2Projective {
            x: Fp2 {
                c0: Fp::from_raw_unchecked([
                    0xf5f28fa202940a10,
                    0xb3f5fb2687b4961a,
                    0xa1a893b53e2ae580,
                    0x9894999d1a3caee9,
                    0x6f67b7631863366b,
                    0x58191924350bcd7,
                ]),
                c1: Fp::from_raw_unchecked([
                    0xa5a9c0759e23f606,
                    0xaaa0c59dbccd60c3,
                    0x3bb17e18e2867806,
                    0x1b1ab6cc8541b367,
                    0xc2b6ed0ef2158547,
                    0x11922a097360edf3,
                ]),
            },
            y: Fp2 {
                c0: Fp::from_raw_unchecked([
                    0x4c730af860494c4a,
                    0x597cfa1f5e369c5a,
                    0xe7e6856caa0a635a,
                    0xbbefb5e96e0d495f,
                    0x7d3a975f0ef25a2,
                    0x83fd8e7e80dae5,
                ]),
                c1: Fp::from_raw_unchecked([
                    0xadc0fc92df64b05d,
                    0x18aa270a2b1461dc,
                    0x86adac6a3be4eba0,
                    0x79495c4ec93da33a,
                    0xe7175850a43ccaed,
                    0xb2bc2a163de1bf2,
                ]),
            },
            z: Fp2::one(),
        }
    }

    /// Computes the doubling of this point.
    pub fn double(&self) -> G2Projective {
        // http://www.hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
        //
        // There are no points of order 2.

        let a = self.x.square();
        let b = self.y.square();
        let c = b.square();
        let d = self.x + b;
        let d = d.square();
        let d = d - a - c;
        let d = d + d;
        let e = a + a + a;
        let f = e.square();
        let z3 = self.z * self.y;
        let z3 = z3 + z3;
        let x3 = f - (d + d);
        let c = c + c;
        let c = c + c;
        let c = c + c;
        let y3 = e * (d - x3) - c;

        let tmp = G2Projective {
            x: x3,
            y: y3,
            z: z3,
        };

        G2Projective::conditional_select(&tmp, &G2Projective::identity(), self.is_identity())
    }

    /// Returns true if this element is the identity (the point at infinity).
    #[inline]
    pub fn is_identity(&self) -> Choice {
        self.z.is_zero()
    }

    /// Returns true if this point is on the curve. This should always return
    /// true unless an "unchecked" API was used.
    pub fn is_on_curve(&self) -> Choice {
        // Y^2 - X^3 = 4(u + 1)(Z^6)

        (self.y.square() - (self.x.square() * self.x))
            .ct_eq(&((self.z.square() * self.z).square() * B))
            | self.z.is_zero()
    }
}

#[test]
fn test_is_on_curve() {
    assert!(bool::from(G2Affine::identity().is_on_curve()));
    assert!(bool::from(G2Affine::generator().is_on_curve()));
    assert!(bool::from(G2Projective::identity().is_on_curve()));
    assert!(bool::from(G2Projective::generator().is_on_curve()));

    let z = Fp2 {
        c0: Fp::from_raw_unchecked([
            0xba7afa1f9a6fe250,
            0xfa0f5b595eafe731,
            0x3bdc477694c306e7,
            0x2149be4b3949fa24,
            0x64aa6e0649b2078c,
            0x12b108ac33643c3e,
        ]),
        c1: Fp::from_raw_unchecked([
            0x125325df3d35b5a8,
            0xdc469ef5555d7fe3,
            0x2d716d2443106a9,
            0x5a1db59a6ff37d0,
            0x7cf7784e5300bb8f,
            0x16a88922c7a5e844,
        ]),
    };

    let gen = G2Affine::generator();
    let mut test = G2Projective {
        x: gen.x * (z.square()),
        y: gen.y * (z.square() * z),
        z,
    };

    assert!(bool::from(test.is_on_curve()));

    test.x = z;
    assert!(!bool::from(test.is_on_curve()));
}

#[test]
fn test_affine_point_equality() {
    let a = G2Affine::generator();
    let b = G2Affine::identity();

    assert!(a == a);
    assert!(b == b);
    assert!(a != b);
    assert!(b != a);
}

#[test]
fn test_projective_point_equality() {
    let a = G2Projective::generator();
    let b = G2Projective::identity();

    assert!(a == a);
    assert!(b == b);
    assert!(a != b);
    assert!(b != a);

    let z = Fp2 {
        c0: Fp::from_raw_unchecked([
            0xba7afa1f9a6fe250,
            0xfa0f5b595eafe731,
            0x3bdc477694c306e7,
            0x2149be4b3949fa24,
            0x64aa6e0649b2078c,
            0x12b108ac33643c3e,
        ]),
        c1: Fp::from_raw_unchecked([
            0x125325df3d35b5a8,
            0xdc469ef5555d7fe3,
            0x2d716d2443106a9,
            0x5a1db59a6ff37d0,
            0x7cf7784e5300bb8f,
            0x16a88922c7a5e844,
        ]),
    };

    let mut c = G2Projective {
        x: a.x * (z.square()),
        y: a.y * (z.square() * z),
        z,
    };
    assert!(bool::from(c.is_on_curve()));

    assert!(a == c);
    assert!(b != c);
    assert!(c == a);
    assert!(c != b);

    c.y = -c.y;
    assert!(bool::from(c.is_on_curve()));

    assert!(a != c);
    assert!(b != c);
    assert!(c != a);
    assert!(c != b);

    c.y = -c.y;
    c.x = z;
    assert!(!bool::from(c.is_on_curve()));
    assert!(a != b);
    assert!(a != c);
    assert!(b != c);
}

#[test]
fn test_conditionally_select_affine() {
    let a = G2Affine::generator();
    let b = G2Affine::identity();

    assert_eq!(G2Affine::conditional_select(&a, &b, Choice::from(0u8)), a);
    assert_eq!(G2Affine::conditional_select(&a, &b, Choice::from(1u8)), b);
}

#[test]
fn test_conditionally_select_projective() {
    let a = G2Projective::generator();
    let b = G2Projective::identity();

    assert_eq!(
        G2Projective::conditional_select(&a, &b, Choice::from(0u8)),
        a
    );
    assert_eq!(
        G2Projective::conditional_select(&a, &b, Choice::from(1u8)),
        b
    );
}

#[test]
fn test_projective_to_affine() {
    let a = G2Projective::generator();
    let b = G2Projective::identity();

    assert!(bool::from(G2Affine::from(a).is_on_curve()));
    assert!(!bool::from(G2Affine::from(a).is_identity()));
    assert!(bool::from(G2Affine::from(b).is_on_curve()));
    assert!(bool::from(G2Affine::from(b).is_identity()));

    let z = Fp2 {
        c0: Fp::from_raw_unchecked([
            0xba7afa1f9a6fe250,
            0xfa0f5b595eafe731,
            0x3bdc477694c306e7,
            0x2149be4b3949fa24,
            0x64aa6e0649b2078c,
            0x12b108ac33643c3e,
        ]),
        c1: Fp::from_raw_unchecked([
            0x125325df3d35b5a8,
            0xdc469ef5555d7fe3,
            0x2d716d2443106a9,
            0x5a1db59a6ff37d0,
            0x7cf7784e5300bb8f,
            0x16a88922c7a5e844,
        ]),
    };

    let c = G2Projective {
        x: a.x * (z.square()),
        y: a.y * (z.square() * z),
        z,
    };

    assert_eq!(G2Affine::from(c), G2Affine::generator());
}

#[test]
fn test_affine_to_projective() {
    let a = G2Affine::generator();
    let b = G2Affine::identity();

    assert!(bool::from(G2Projective::from(a).is_on_curve()));
    assert!(!bool::from(G2Projective::from(a).is_identity()));
    assert!(bool::from(G2Projective::from(b).is_on_curve()));
    assert!(bool::from(G2Projective::from(b).is_identity()));
}

#[test]
fn test_doubling() {
    {
        let tmp = G2Projective::identity().double();
        assert!(bool::from(tmp.is_identity()));
        assert!(bool::from(tmp.is_on_curve()));
    }
    {
        let tmp = G2Projective::generator().double();
        assert!(!bool::from(tmp.is_identity()));
        assert!(bool::from(tmp.is_on_curve()));

        assert_eq!(
            G2Affine::from(tmp),
            G2Affine {
                x: Fp2 {
                    c0: Fp::from_raw_unchecked([
                        0xe9d9e2da9620f98b,
                        0x54f1199346b97f36,
                        0x3db3b820376bed27,
                        0xcfdb31c9b0b64f4c,
                        0x41d7c12786354493,
                        0x5710794c255c064
                    ]),
                    c1: Fp::from_raw_unchecked([
                        0xd6c1d3ca6ea0d06e,
                        0xda0cbd905595489f,
                        0x4f5352d43479221d,
                        0x8ade5d736f8c97e0,
                        0x48cc8433925ef70e,
                        0x8d7ea71ea91ef81
                    ]),
                },
                y: Fp2 {
                    c0: Fp::from_raw_unchecked([
                        0x15ba26eb4b0d186f,
                        0xd086d64b7e9e01e,
                        0xc8b848dd652f4c78,
                        0xeecf46a6123bae4f,
                        0x255e8dd8b6dc812a,
                        0x164142af21dcf93f
                    ]),
                    c1: Fp::from_raw_unchecked([
                        0xf9b4a1a895984db4,
                        0xd417b114cccff748,
                        0x6856301fc89f086e,
                        0x41c777878931e3da,
                        0x3556b155066a2105,
                        0xacf7d325cb89cf
                    ]),
                },
                infinity: Choice::from(0u8)
            }
        );
    }
}
