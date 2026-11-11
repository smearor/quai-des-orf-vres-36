//! `BlockFaces` - the set of faces in a block layout.

use crate::Segment;
use crate::map::DistrictName;
use crate::map::Park;
use crate::map::backdrop::Landmark;
use crate::map::blocks::BlockFace;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use std::ops::Deref;
use std::ops::DerefMut;

/// The faces of a block layout as a set: tunnels attach, and
/// district names pin and fill, on the whole collection.
/// Derefs to the `[BlockFace]` slice.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockFaces(pub Vec<BlockFace>);

impl BlockFaces {
    /// Attaches every segment in `tunnels` to each face it
    /// touches (see [`BlockFace::attach_tunnel`]).
    pub fn attach_all(&mut self, tunnels: impl IntoIterator<Item = Segment<Millimeters>>) {
        for segment in tunnels {
            for face in self.0.iter_mut() {
                face.attach_tunnel(segment);
            }
        }
    }

    /// Names the still-unnamed inner face containing `pos` after
    /// `district` - the face pins the positioned quarter's name
    /// (outer faces never carry one).
    pub fn pin_district(&mut self, pos: &Vec2<Millimeters>, name: &DistrictName) {
        if let Some(face) = self.0.iter_mut().find(|face| !face.is_outer() && face.district.is_none() && face.contains(pos)) {
            face.district = Some(name.clone());
        }
    }

    /// Names each still-unnamed inner face after the next name
    /// in `names`, in order - the fallback for quarters without
    /// a dataset position.
    pub fn fill_districts(&mut self, names: impl IntoIterator<Item = DistrictName>) {
        let mut names = names.into_iter();
        for face in self.0.iter_mut().filter(|face| !face.is_outer() && face.district.is_none()) {
            let Some(name) = names.next() else { break };
            face.district = Some(name);
        }
    }

    /// Parks and landmarks land in the inner face containing
    /// them.
    pub fn populate(&mut self, parks: &[Park], landmarks: &[Landmark]) {
        for face in self.inner_mut() {
            face.parks = parks.iter().filter(|park| face.contains(&park.pos)).cloned().collect();
            face.landmarks = landmarks.iter().filter(|landmark| face.contains(&landmark.pos)).cloned().collect();
        }
    }

    /// The inner (non-outer) faces.
    pub fn inner(&self) -> impl Iterator<Item = &BlockFace> {
        self.0.iter().filter(|face| !face.is_outer())
    }

    /// The inner (non-outer) faces, mutably.
    pub fn inner_mut(&mut self) -> impl Iterator<Item = &mut BlockFace> {
        self.0.iter_mut().filter(|face| !face.is_outer())
    }
}

impl Deref for BlockFaces {
    type Target = [BlockFace];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BlockFaces {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a BlockFaces {
    type Item = &'a BlockFace;
    type IntoIter = std::slice::Iter<'a, BlockFace>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
