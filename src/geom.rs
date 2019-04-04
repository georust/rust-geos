use crate::{CoordSeq, GContextHandle};
use enums::*;
use error::{Error, GResult, PredicateType};
use ffi::*;
use functions::*;
use libc::{c_double, c_int, c_uint};
use std::ffi::CString;
use std::ptr::NonNull;
use std::{self, mem, str};
use c_vec::CVec;
use std::sync::Arc;

pub struct GGeom<'a> {
    ptr: NonNull<GEOSGeometry>,
    context: Arc<GContextHandle<'a>>,
}

impl<'a> GGeom<'a> {
    /// Create a new [`GGeom`] from the WKT format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// ```
    pub fn new_from_wkt(wkt: &str) -> GResult<GGeom<'a>> {
        initialize();
        match GContextHandle::init() {
            Ok(context_handle) => {
                let c_str = CString::new(wkt).expect("Conversion to CString failed");
                unsafe {
                    let reader = GEOSWKTReader_create_r(context_handle.as_raw());
                    let obj = GEOSWKTReader_read_r(context_handle.as_raw(), reader, c_str.as_ptr());
                    GEOSWKTReader_destroy_r(context_handle.as_raw(), reader);
                    if obj.is_null() {
                        return Err(Error::NoConstructionFromNullPtr);
                    }
                    GGeom::new_from_raw(obj, Arc::new(context_handle))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Create a new [`GGeom`] from the HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_hex().expect("conversion to HEX failed");
    ///
    /// // The interesting part is here:
    /// let new_geom = GGeom::new_from_hex(hex_buf.as_ref())
    ///                      .expect("conversion from HEX failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn new_from_hex(hex: &[u8]) -> GResult<GGeom<'a>> {
        initialize();
        match GContextHandle::init() {
            Ok(context) => {
                unsafe {
                    let ptr = GEOSGeomFromHEX_buf_r(context.as_raw(), hex.as_ptr(), hex.len());
                    GGeom::new_from_raw(ptr, Arc::new(context))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Create a new [`GGeom`] from the WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = point_geom.to_wkb().expect("conversion to WKB failed");
    ///
    /// // The interesting part is here:
    /// let new_geom = GGeom::new_from_wkb(wkb_buf.as_ref())
    ///                      .expect("conversion from WKB failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn new_from_wkb(wkb: &[u8]) -> GResult<GGeom<'a>> {
        initialize();
        match GContextHandle::init() {
            Ok(context) => {
                unsafe {
                    let ptr = GEOSGeomFromWKB_buf_r(context.as_raw(), wkb.as_ptr(), wkb.len());
                    GGeom::new_from_raw(ptr, Arc::new(context))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Converts a [`GGeom`] to the HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_hex().expect("conversion to WKB failed");
    /// ```
    pub fn to_hex(&self) -> Option<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToHEX_buf_r(self.context.as_raw(), self.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }

    /// Converts a [`GGeom`] to the WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_wkb().expect("conversion to WKB failed");
    /// ```
    pub fn to_wkb(&self) -> Option<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToWKB_buf_r(self.context.as_raw(), self.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }

    /// Set the context handle to the geometry.
    ///
    /// ```
    /// use geos::{GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// let mut point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// point_geom.set_context_handle(context_handle);
    /// ```
    pub fn set_context_handle(&mut self, context: GContextHandle<'a>) {
        self.context = Arc::new(context);
    }

    /// Get the context handle of the geometry.
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let context = point_geom.get_context_handle();
    /// context.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    pub fn get_context_handle(&self) -> &GContextHandle<'a> {
        &self.context
    }

    pub(crate) unsafe fn new_from_raw(
        g: *mut GEOSGeometry,
        context: Arc<GContextHandle<'a>>,
    ) -> GResult<GGeom<'a>> {
        NonNull::new(g)
            .ok_or(Error::NoConstructionFromNullPtr)
            .map(|ptr| GGeom { ptr, context })
    }

    pub(crate) fn as_raw(&self) -> &GEOSGeometry {
        unsafe { self.ptr.as_ref() }
    }

    pub(crate) fn as_raw_mut(&mut self) -> &mut GEOSGeometry {
        unsafe { self.ptr.as_mut() }
    }

    pub(crate) fn clone_context(&self) -> Arc<GContextHandle<'a>> {
        Arc::clone(&self.context)
    }

    pub fn is_valid(&self) -> bool {
        unsafe { GEOSisValid_r(self.context.as_raw(), self.as_raw()) == 1 }
    }

    /// Get the underlying geos CoordSeq object from the geometry
    ///
    /// Note: this clones the underlying CoordSeq to avoid double free
    /// (because CoordSeq handles the object ptr and the CoordSeq is still owned by the geos geometry)
    /// if this method's performance becomes a bottleneck, feel free to open an issue, we could skip this clone with cleaner code
    pub fn get_coord_seq(&self) -> Result<CoordSeq, Error> {
        let type_geom = self.geometry_type();
        match type_geom {
            GGeomTypes::Point | GGeomTypes::LineString | GGeomTypes::LinearRing => unsafe {
                let t = GEOSCoordSeq_clone(GEOSGeom_getCoordSeq(self.as_raw()));
                CoordSeq::new_from_raw(t)
            },
            _ => Err(Error::ImpossibleOperation(
                "Geometry must be a Point, LineString or LinearRing to extract it's coordinates"
                    .into(),
            )),
        }
    }

    pub fn geometry_type(&self) -> GGeomTypes {
        let type_geom = unsafe { GEOSGeomTypeId_r(self.context.as_raw(), self.as_raw()) as i32 };

        GGeomTypes::from(type_geom)
    }

    pub fn area(&self) -> GResult<f64> {
        let mut n = 0.;

        let res = unsafe { GEOSArea_r(self.context.as_raw(), self.as_raw(), &mut n) };
        if res != 1 {
            Err(Error::GeosError(format!("area failed with code {}", res)))
        } else {
            Ok(n as f64)
        }
    }

    pub fn to_wkt(&self) -> String {
        unsafe { managed_string(GEOSGeomToWKT_r(self.context.as_raw(), self.as_raw())) }
    }

    pub fn to_wkt_precision(&self, precision: Option<u32>) -> String {
        unsafe {
            let writer = GEOSWKTWriter_create_r(self.context.as_raw());
            if let Some(x) = precision {
                GEOSWKTWriter_setRoundingPrecision_r(self.context.as_raw(), writer, x as c_int)
            };
            let c_result = GEOSWKTWriter_write_r(self.context.as_raw(), writer, self.as_raw());
            GEOSWKTWriter_destroy_r(self.context.as_raw(), writer);
            managed_string(c_result)
        }
    }

    pub fn is_ring(&self) -> GResult<bool> {
        let rv = unsafe { GEOSisRing_r(self.context.as_raw(), self.as_raw()) };
        check_geos_predicate(rv as _, PredicateType::IsRing)
    }

    pub fn intersects<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSIntersects_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Intersects)
    }

    pub fn crosses<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSCrosses_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Crosses)
    }

    pub fn disjoint<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSDisjoint_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Disjoint)
    }

    pub fn touches<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSTouches_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Touches)
    }

    pub fn overlaps<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSOverlaps_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Overlaps)
    }

    pub fn within<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSWithin_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Within)
    }

    /// Checks if the two [`GGeom`] objects are equal.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let geom1 = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let geom2 = GGeom::new_from_wkt("POINT (3.8 3.8)").expect("Invalid geometry");
    /// let geom3 = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert!(geom1.equals(&geom2) == Ok(false));
    /// assert!(geom1.equals(&geom3) == Ok(true));
    /// ```
    ///
    /// Note that you can also use method through the `PartialEq` trait:
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let geom1 = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let geom2 = GGeom::new_from_wkt("POINT (3.8 3.8)").expect("Invalid geometry");
    /// let geom3 = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert!(geom1 != geom2);
    /// assert!(geom1 == geom3);
    /// ```
    pub fn equals<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSEquals_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Equals)
    }

    pub fn equals_exact<'b>(&self, g2: &GGeom<'b>, precision: f64) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSEqualsExact_r(self.context.as_raw(), self.as_raw(), g2.as_raw(), precision)
        };
        check_geos_predicate(ret_val as _, PredicateType::EqualsExact)
    }

    pub fn covers<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSCovers_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Covers)
    }

    pub fn covered_by<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSCoveredBy_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::CoveredBy)
    }

    pub fn contains<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSContains_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Contains)
    }

    pub fn buffer(&self, width: f64, quadsegs: i32) -> GResult<GGeom<'a>> {
        assert!(quadsegs > 0);
        unsafe {
            let ptr = GEOSBuffer_r(
                self.context.as_raw(),
                self.as_raw(),
                width as c_double,
                quadsegs as c_int,
            );
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn is_empty(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisEmpty_r(self.context.as_raw(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsEmpty)
    }

    pub fn is_simple(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisSimple_r(self.context.as_raw(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn difference<'b>(&self, g2: &GGeom<'b>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSDifference_r(self.context.as_raw(), self.as_raw(), g2.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn envelope(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSEnvelope_r(self.context.as_raw(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn sym_difference<'b>(&self, g2: &GGeom<'b>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSSymDifference_r(self.context.as_raw(), self.as_raw(), g2.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn union(&self, g2: &GGeom<'a>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSUnion_r(self.context.as_raw(), self.as_raw(), g2.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn get_centroid(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSGetCentroid_r(self.context.as_raw(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn create_polygon<'b>(mut exterior: GGeom<'a>, mut interiors: Vec<GGeom<'b>>) -> GResult<GGeom<'a>> {
        let context_handle = exterior.clone_context();
        let nb_interiors = interiors.len();
        let res = unsafe {
            let ptr = GEOSGeom_createPolygon_r(
                context_handle.as_raw(),
                exterior.ptr.as_mut(),
                interiors.as_mut_ptr() as *mut *mut GEOSGeometry,
                nb_interiors as c_uint,
            );
            GGeom::new_from_raw(ptr, context_handle)
        };

        // we'll transfert the ownership of the ptr to the new GGeom,
        // so the old one needs to forget their c ptr to avoid double cleanup
        mem::forget(exterior);
        for i in interiors {
            mem::forget(i);
        }

        res
    }

    pub fn create_geometrycollection(geoms: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        create_multi_geom(geoms, GGeomTypes::GeometryCollection)
    }

    pub fn create_multipolygon(polygons: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        if !check_same_geometry_type(&polygons, GGeomTypes::Polygon) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Polygon".to_string(),
            ));
        }
        create_multi_geom(polygons, GGeomTypes::MultiPolygon)
    }

    pub fn create_multilinestring(linestrings: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        if !check_same_geometry_type(&linestrings, GGeomTypes::LineString) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type LineString".to_string(),
            ));
        }
        create_multi_geom(linestrings, GGeomTypes::MultiLineString)
    }

    pub fn create_multipoint(points: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        if !check_same_geometry_type(&points, GGeomTypes::Point) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Point".to_string(),
            ));
        }
        create_multi_geom(points, GGeomTypes::MultiPoint)
    }

    pub fn create_point(s: CoordSeq) -> GResult<GGeom<'a>> {
        match GContextHandle::init() {
            Ok(context_handle) => {
                unsafe {
                    // FIXME: is cloning really necessary?
                    let coords = GEOSCoordSeq_clone_r(context_handle.as_raw(), s.as_raw());
                    let ptr = GEOSGeom_createPoint_r(context_handle.as_raw(), coords);
                    GGeom::new_from_raw(ptr, Arc::new(context_handle))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn create_line_string(s: CoordSeq) -> GResult<GGeom<'a>> {
        match GContextHandle::init() {
            Ok(context_handle) => {
                unsafe {
                    // FIXME: Should we clone line in `create_point`?
                    let ptr = GEOSGeom_createLineString_r(context_handle.as_raw(), s.as_raw());
                    mem::forget(s);
                    GGeom::new_from_raw(ptr, Arc::new(context_handle))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn create_linear_ring(s: CoordSeq) -> GResult<GGeom<'a>> {
        match GContextHandle::init() {
            Ok(context_handle) => {
                unsafe {
                    // FIXME: Should we clone line in `create_point`?
                    let ptr = GEOSGeom_createLinearRing_r(context_handle.as_raw(), s.as_raw());
                    mem::forget(s);
                    GGeom::new_from_raw(ptr, Arc::new(context_handle))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn unary_union(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSUnaryUnion_r(self.context.as_raw(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn voronoi<'b>(
        &self,
        envelope: Option<&GGeom<'b>>,
        tolerance: f64,
        only_edges: bool,
    ) -> GResult<GGeom<'a>> {
        unsafe {
            let raw_voronoi = GEOSVoronoiDiagram_r(
                self.context.as_raw(),
                self.as_raw(),
                envelope
                    .map(|e| e.ptr.as_ptr() as *const GEOSGeometry)
                    .unwrap_or(std::ptr::null()),
                tolerance,
                only_edges as c_int,
            );
            Self::new_from_raw(raw_voronoi, self.clone_context())
        }
    }

    pub fn normalize(&mut self) -> GResult<bool> {
        let ret_val = unsafe { GEOSNormalize_r(self.context.as_raw(), self.as_raw_mut()) };
        check_geos_predicate(ret_val, PredicateType::Normalize)
    }

    pub fn intersection<'b>(&self, other: &GGeom<'b>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSIntersection_r(self.context.as_raw(), self.as_raw(), other.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn convex_hull(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSConvexHull_r(self.context.as_raw(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn boundary(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSBoundary_r(self.context.as_raw(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn has_z(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSHasZ_r(self.context.as_raw(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn is_closed(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisClosed_r(self.context.as_raw(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn length(&self) -> GResult<f64> {
        let mut length = 0.;
        unsafe {
            let ret = GEOSLength_r(self.context.as_raw(), self.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    pub fn distance<'b>(&self, other: &GGeom<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistance_r(
                self.context.as_raw(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn distance_indexed<'b>(&self, other: &GGeom<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistanceIndexed_r(
                self.context.as_raw(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn hausdorff_distance<'b>(&self, other: &GGeom<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistance_r(
                self.context.as_raw(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn hausdorff_distance_densify<'b>(&self, other: &GGeom<'b>, distance_frac: f64) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistanceDensify_r(
                self.context.as_raw(),
                self.as_raw(),
                other.as_raw(),
                distance_frac,
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn frechet_distance<'b>(&self, other: &GGeom<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistance_r(
                self.context.as_raw(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn frechet_distance_densify<'b>(&self, other: &GGeom<'b>, distance_frac: f64) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistanceDensify_r(
                self.context.as_raw(),
                self.as_raw(),
                other.as_raw(),
                distance_frac,
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn get_length(&self) -> GResult<f64> {
        let mut length = 0.;
        unsafe {
            let ret = GEOSGeomGetLength_r(self.context.as_raw(), self.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    pub fn snap<'b>(&self, other: &GGeom<'b>, tolerance: f64) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSSnap_r(self.context.as_raw(), self.as_raw(), other.as_raw(), tolerance);
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn extract_unique_points(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSGeom_extractUniquePoints_r(self.context.as_raw(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn nearest_points<'b>(&self, other: &GGeom<'b>) -> GResult<CoordSeq> {
        unsafe {
            CoordSeq::new_from_raw(GEOSNearestPoints_r(
                self.context.as_raw(),
                self.as_raw(),
                other.as_raw()))
        }
    }
}

unsafe impl<'a> Send for GGeom<'a> {}
unsafe impl<'a> Sync for GGeom<'a> {}

impl<'a> Drop for GGeom<'a> {
    fn drop(&mut self) {
        unsafe { GEOSGeom_destroy_r(self.context.as_raw(), self.as_raw_mut()) }
    }
}

impl<'a> Clone for GGeom<'a> {
    /// Also pass the context to the newly created `GGeom`.
    fn clone(&self) -> GGeom<'a> {
        let ptr = unsafe { GEOSGeom_clone_r(self.context.as_raw(), self.as_raw()) };
        GGeom {
            ptr: NonNull::new(ptr).unwrap(),
            context: self.clone_context(),
        }
    }
}

impl<'a> PartialEq for GGeom<'a> {
    fn eq<'b>(&self, other: &GGeom<'b>) -> bool {
        self.equals(other).unwrap_or_else(|_| false)
    }
}
