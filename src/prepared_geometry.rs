use crate::{AsRaw, ContextHandle, ContextHandling, ContextInteractions, GResult, Geom};
use context_handle::PtrWrap;
use error::Error;
use error::PredicateType;
use functions::*;
use geos_sys::*;

use std::mem::transmute;
use std::sync::Arc;

/// `PreparedGeometry` is an interface which prepares [`Geometry`] for greater performance
/// on repeated calls.
///
/// # Example
///
/// ```
/// use geos::{Geom, Geometry};
///
/// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
///                      .expect("Invalid geometry");
/// let mut prepared_geom = geom1.to_prepared_geom()
///                              .expect("failed to create prepared geom");
/// let geom2 = Geometry::new_from_wkt("POINT (2.5 2.5)")
///                      .expect("Invalid geometry");
///
/// assert_eq!(prepared_geom.contains(&geom2), Ok(true));
/// ```
pub struct PreparedGeometry<'a> {
    ptr: PtrWrap<*const GEOSPreparedGeometry>,
    context: Arc<ContextHandle<'a>>,
}

impl<'a> PreparedGeometry<'a> {
    /// Creates a new `PreparedGeometry` from a [`Geometry`].
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, PreparedGeometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                      .expect("Invalid geometry");
    /// let prepared_geom = PreparedGeometry::new(&geom1);
    /// ```
    pub fn new<'b: 'a, G: Geom<'b>>(g: &'a G) -> GResult<PreparedGeometry<'a>> {
        unsafe {
            let ptr = GEOSPrepare_r(g.get_raw_context(), g.as_raw());
            PreparedGeometry::new_from_raw(ptr, transmute(g.clone_context()), "new")
        }
    }

    pub(crate) unsafe fn new_from_raw(
        ptr: *const GEOSPreparedGeometry,
        context: Arc<ContextHandle<'a>>,
        caller: &str,
    ) -> GResult<PreparedGeometry<'a>> {
        if ptr.is_null() {
            let extra = if let Some(x) = context.get_last_error() {
                format!("\nLast error: {}", x)
            } else {
                String::new()
            };
            return Err(Error::NoConstructionFromNullPtr(format!(
                "PreparedGeometry::{}{}",
                caller, extra
            )));
        }
        Ok(PreparedGeometry {
            ptr: PtrWrap(ptr),
            context,
        })
    }

    /// Returns `true` if no points of the other geometry is outside the exterior of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                      .expect("Invalid geometry");
    /// let mut prepared_geom = geom1
    ///     .to_prepared_geom()
    ///     .expect("failed to create prepared geom");
    /// let geom2 = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                      .expect("Invalid geometry");
    ///
    /// assert_eq!(prepared_geom.contains(&geom2), Ok(true));
    /// ```
    pub fn contains<'b, G: Geom<'b>>(&self, other: &G) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedContains_r(self.get_raw_context(), self.as_raw(), other.as_raw())
        };
        check_geos_predicate(ret_val as _, PredicateType::PreparedContains)
    }

    /// Returns `true` if every point of the `other` geometry is inside self's interior.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                      .expect("Invalid geometry");
    /// let mut prepared_geom = geom1
    ///     .to_prepared_geom()
    ///     .expect("failed to create prepared geom");
    /// let geom2 = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                      .expect("Invalid geometry");
    ///
    /// assert_eq!(prepared_geom.contains_properly(&geom2), Ok(true));
    /// ```
    pub fn contains_properly<'b, G: Geom<'b>>(&self, other: &G) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedContainsProperly_r(self.get_raw_context(), self.as_raw(), other.as_raw())
        };
        check_geos_predicate(ret_val as _, PredicateType::PreparedContainsProperly)
    }

    /// Returns `true` if no point of `self` is outside of `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT (1 2)")
    ///                     .expect("Invalid geometry");
    /// let little_geom = geom.buffer(10., 8).expect("buffer failed");
    /// let big_geom = geom.buffer(20., 8).expect("buffer failed");
    ///
    /// let prepared_little_geom = little_geom
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    /// let prepared_big_geom = big_geom
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    ///
    /// assert_eq!(prepared_little_geom.covered_by(&big_geom), Ok(true));
    /// assert_eq!(prepared_big_geom.covered_by(&little_geom), Ok(false));
    /// ```
    pub fn covered_by<'b, G: Geom<'b>>(&self, other: &G) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedCoveredBy_r(self.get_raw_context(), self.as_raw(), other.as_raw())
        };
        check_geos_predicate(ret_val as _, PredicateType::PreparedCoveredBy)
    }

    /// Returns `true` if no point of `other` is outside of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT (1 2)")
    ///                     .expect("Invalid geometry");
    /// let little_geom = geom.buffer(10., 8).expect("buffer failed");
    /// let big_geom = geom.buffer(20., 8).expect("buffer failed");
    ///
    /// let prepared_little_geom = little_geom
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    /// let prepared_big_geom = big_geom
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    ///
    /// assert_eq!(prepared_little_geom.covers(&big_geom), Ok(false));
    /// assert_eq!(prepared_big_geom.covers(&little_geom), Ok(true));
    /// ```
    pub fn covers<'b, G: Geom<'b>>(&self, other: &G) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSPreparedCovers_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::PreparedCovers)
    }

    /// Returns `true` if `self` and `other` have at least one interior into each other.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(1 1,2 2)")
    ///                      .expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 1,1 2)")
    ///                      .expect("invalid geometry");
    /// let prepared_geom = geom1.to_prepared_geom().expect("to_prepared_geom failed");
    ///
    /// assert_eq!(prepared_geom.crosses(&geom2), Ok(true));
    /// ```
    pub fn crosses<'b, G: Geom<'b>>(&self, other: &G) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSPreparedCrosses_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::PreparedCrosses)
    }

    /// Returns `true` if `self` doesn't:
    ///
    /// * Overlap `other`
    /// * Touch `other`
    /// * Is within `other`
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(0 0)")
    ///                      .expect("invalid geometry");
    /// let prepared_geom = geom1
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 0, 0 2)")
    ///                      .expect("invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("LINESTRING(0 0, 0 2)")
    ///                      .expect("invalid geometry");
    ///
    /// assert_eq!(prepared_geom.disjoint(&geom2), Ok(true));
    /// assert_eq!(prepared_geom.disjoint(&geom3), Ok(false));
    /// ```
    pub fn disjoint<'b, G: Geom<'b>>(&self, other: &G) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedDisjoint_r(self.get_raw_context(), self.as_raw(), other.as_raw())
        };
        check_geos_predicate(ret_val as _, PredicateType::PreparedDisjoint)
    }

    /// Returns `true` if `self` shares any portion of space with `other`. So if any of this is
    /// `true`:
    ///
    /// * `self` overlaps `other`
    /// * `self` touches `other`
    /// * `self` is within `other`
    ///
    /// Then `intersects` will return `true` as well.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(0 0)")
    ///                      .expect("invalid geometry");
    /// let prepared_geom = geom1
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 0, 0 2)")
    ///                      .expect("invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("LINESTRING(0 0, 0 2)")
    ///                      .expect("invalid geometry");
    ///
    /// assert_eq!(prepared_geom.intersects(&geom2), Ok(false));
    /// assert_eq!(prepared_geom.intersects(&geom3), Ok(true));
    /// ```
    pub fn intersects<'b, G: Geom<'b>>(&self, other: &G) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedIntersects_r(self.get_raw_context(), self.as_raw(), other.as_raw())
        };
        check_geos_predicate(ret_val as _, PredicateType::PreparedIntersects)
    }

    /// Returns `true` if `self` spatially overlaps `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(1 0.5)")
    ///                      .expect("invalid geometry");
    /// let prepared_geom = geom1
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(1 0, 1 1, 3 5)")
    ///                      .expect("invalid geometry");
    ///
    /// assert_eq!(prepared_geom.overlaps(&geom2), Ok(false));
    ///
    /// let geom1 = geom1.buffer(3., 8).expect("buffer failed");
    /// let prepared_geom = geom1
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    /// let geom2 = geom2.buffer(0.5, 8).expect("buffer failed");
    ///
    /// assert_eq!(prepared_geom.overlaps(&geom2), Ok(true));
    /// ```
    pub fn overlaps<'b, G: Geom<'b>>(&self, other: &G) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedOverlaps_r(self.get_raw_context(), self.as_raw(), other.as_raw())
        };
        check_geos_predicate(ret_val as _, PredicateType::PreparedOverlaps)
    }

    /// Returns `true` if the only points in common between `self` and `other` lie in the union of
    /// the boundaries of `self` and `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(0 0, 1 1, 0 2)")
    ///                      .expect("invalid geometry");
    /// let prepared_geom = geom1
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    /// let geom2 = Geometry::new_from_wkt("POINT(1 1)").expect("invalid geometry");
    ///
    /// assert_eq!(prepared_geom.touches(&geom2), Ok(false));
    ///
    /// let geom2 = Geometry::new_from_wkt("POINT(0 2)").expect("invalid geometry");
    ///
    /// assert_eq!(prepared_geom.touches(&geom2), Ok(true));
    /// ```
    pub fn touches<'b, G: Geom<'b>>(&self, other: &G) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSPreparedTouches_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::PreparedTouches)
    }

    /// Returns `true` if `self` is completely inside `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT(50 50)")
    ///                     .expect("invalid geometry");
    /// let small_geom = geom.buffer(20., 8).expect("buffer failed");
    /// let big_geom = geom.buffer(40., 8).expect("buffer failed");
    ///
    /// let small_prepared_geom = small_geom
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    /// let big_prepared_geom = big_geom
    ///     .to_prepared_geom()
    ///     .expect("to_prepared_geom failed");
    ///
    /// assert_eq!(small_prepared_geom.within(&small_geom), Ok(true));
    /// assert_eq!(small_prepared_geom.within(&big_geom), Ok(true));
    /// assert_eq!(big_prepared_geom.within(&small_geom), Ok(false));
    /// ```
    pub fn within<'b, G: Geom<'b>>(&self, other: &G) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSPreparedWithin_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::PreparedWithin)
    }
}

unsafe impl<'a> Send for PreparedGeometry<'a> {}
unsafe impl<'a> Sync for PreparedGeometry<'a> {}

impl<'a> Drop for PreparedGeometry<'a> {
    fn drop(&mut self) {
        unsafe { GEOSPreparedGeom_destroy_r(self.get_raw_context(), self.as_raw()) };
    }
}

impl<'a> ContextInteractions<'a> for PreparedGeometry<'a> {
    /// Set the context handle to the `PreparedGeometry`.
    ///
    /// ```
    /// use geos::{
    ///     ContextInteractions, ContextHandle, Geom, Geometry, PreparedGeometry,
    /// };
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                           .expect("Invalid geometry");
    /// let context_handle = ContextHandle::init().expect("invalid init");
    /// let mut prepared_geom = point_geom
    ///     .to_prepared_geom()
    ///     .expect("failed to create prepared geom");
    /// context_handle.set_notice_message_handler(
    ///     Some(Box::new(|s| println!("new message: {}", s)))
    /// );
    /// prepared_geom.set_context_handle(context_handle);
    /// ```
    fn set_context_handle(&mut self, context: ContextHandle<'a>) {
        self.context = Arc::new(context);
    }

    /// Get the context handle of the `PreparedGeometry`.
    ///
    /// ```
    /// use geos::{
    ///     ContextInteractions, CoordDimensions, Geom, Geometry,
    ///     PreparedGeometry,
    /// };
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                           .expect("Invalid geometry");
    /// let prepared_geom = point_geom.to_prepared_geom()
    ///                               .expect("failed to create prepared geom");
    /// let context = prepared_geom.get_context_handle();
    /// context.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    fn get_context_handle(&self) -> &ContextHandle<'a> {
        &self.context
    }
}

impl<'a> AsRaw for PreparedGeometry<'a> {
    type RawType = GEOSPreparedGeometry;

    fn as_raw(&self) -> *const Self::RawType {
        *self.ptr
    }
}

impl<'a> ContextHandling for PreparedGeometry<'a> {
    type Context = Arc<ContextHandle<'a>>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.context.as_raw()
    }

    fn clone_context(&self) -> Arc<ContextHandle<'a>> {
        Arc::clone(&self.context)
    }
}

/// Tests to ensure that the lifetime is correctly set.
///
/// ```compile_fail
/// use geos::{Geom, Geometry, PreparedGeometry};
///
/// pub struct Boo {
///     #[allow(dead_code)]
///     geom: Geometry<'static>,
///     pub prep: PreparedGeometry<'static>,
/// }
/// let boo = {
///     let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
///         .expect("Invalid geometry");
///     let prep = geom
///         .to_prepared_geom()
///         .expect("failed to create prepared geom");
///      Boo { geom, prep }
/// };
/// let pt = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
/// assert!(boo.prep.contains(&pt).unwrap());
/// ```
///
/// ```compile_fail
/// use geos::{Geom, Geometry, PreparedGeometry};
///
/// pub struct Boo {
///     pub prep: PreparedGeometry<'static>,
/// }
///
/// let boo = {
///     let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
///         .expect("Invalid geometry");
///     let prep = geom1
///         .to_prepared_geom()
///         .expect("failed to create prepared geom");
///
///     Boo { prep }
/// };
///
/// let pt = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
///
/// assert!(boo.prep.contains(&pt).unwrap());
/// ```
#[cfg(doctest)]
pub mod lifetime_checks {}
