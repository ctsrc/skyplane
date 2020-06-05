/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    id::{BindGroupLayoutId, BufferId, DeviceId, SamplerId, TextureViewId},
    track::{TrackerSet, DUMMY_SELECTOR},
    FastHashMap, LifeGuard, RefCount, Stored, MAX_BIND_GROUPS,
};

use arrayvec::ArrayVec;
use gfx_descriptor::{DescriptorCounts, DescriptorSet};

#[cfg(feature = "replay")]
use serde::Deserialize;
#[cfg(feature = "trace")]
use serde::Serialize;
use std::borrow::Borrow;

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "trace", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum BindingType {
    UniformBuffer = 0,
    StorageBuffer = 1,
    ReadonlyStorageBuffer = 2,
    Sampler = 3,
    ComparisonSampler = 4,
    SampledTexture = 5,
    ReadonlyStorageTexture = 6,
    WriteonlyStorageTexture = 7,
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq)]
#[cfg_attr(feature = "trace", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct BindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: wgt::ShaderStage,
    pub ty: BindingType,
    pub multisampled: bool,
    pub has_dynamic_offset: bool,
    pub view_dimension: wgt::TextureViewDimension,
    pub texture_component_type: wgt::TextureComponentType,
    pub storage_texture_format: wgt::TextureFormat,
}

#[derive(Clone, Debug)]
pub enum BindGroupLayoutEntryError {
    NoVisibility,
    UnexpectedHasDynamicOffset,
    UnexpectedMultisampled,
}

impl BindGroupLayoutEntry {
    pub(crate) fn validate(&self) -> Result<(), BindGroupLayoutEntryError> {
        if self.visibility.is_empty() {
            return Err(BindGroupLayoutEntryError::NoVisibility);
        }
        match self.ty {
            BindingType::UniformBuffer | BindingType::StorageBuffer => {}
            _ => {
                if self.has_dynamic_offset {
                    return Err(BindGroupLayoutEntryError::UnexpectedHasDynamicOffset);
                }
            }
        }
        match self.ty {
            BindingType::SampledTexture => {}
            _ => {
                if self.multisampled {
                    return Err(BindGroupLayoutEntryError::UnexpectedMultisampled);
                }
            }
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct BindGroupLayoutDescriptor {
    pub label: *const std::os::raw::c_char,
    pub entries: *const BindGroupLayoutEntry,
    pub entries_length: usize,
}

#[derive(Clone, Debug)]
pub enum BindGroupLayoutError {
    ConflictBinding(u32),
    Entry(u32, BindGroupLayoutEntryError),
}

pub(crate) type BindEntryMap = FastHashMap<u32, BindGroupLayoutEntry>;

#[derive(Debug)]
pub struct BindGroupLayout<B: hal::Backend> {
    pub(crate) raw: B::DescriptorSetLayout,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) life_guard: LifeGuard,
    pub(crate) entries: BindEntryMap,
    pub(crate) desc_counts: DescriptorCounts,
    pub(crate) dynamic_count: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct PipelineLayoutDescriptor {
    pub bind_group_layouts: *const BindGroupLayoutId,
    pub bind_group_layouts_length: usize,
}

#[derive(Clone, Debug)]
pub enum PipelineLayoutError {
    TooManyGroups(usize),
}

#[derive(Debug)]
pub struct PipelineLayout<B: hal::Backend> {
    pub(crate) raw: B::PipelineLayout,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) life_guard: LifeGuard,
    pub(crate) bind_group_layout_ids: ArrayVec<[Stored<BindGroupLayoutId>; MAX_BIND_GROUPS]>,
}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "trace", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct BufferBinding {
    pub buffer: BufferId,
    pub offset: wgt::BufferAddress,
    pub size: wgt::BufferSize,
}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "trace", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum BindingResource {
    Buffer(BufferBinding),
    Sampler(SamplerId),
    TextureView(TextureViewId),
}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "trace", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct BindGroupEntry {
    pub binding: u32,
    pub resource: BindingResource,
}

#[repr(C)]
#[derive(Debug)]
pub struct BindGroupDescriptor {
    pub label: *const std::os::raw::c_char,
    pub layout: BindGroupLayoutId,
    pub entries: *const BindGroupEntry,
    pub entries_length: usize,
}

#[derive(Debug)]
pub struct BindGroup<B: hal::Backend> {
    pub(crate) raw: DescriptorSet<B>,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) layout_id: BindGroupLayoutId,
    pub(crate) life_guard: LifeGuard,
    pub(crate) used: TrackerSet,
    pub(crate) dynamic_count: usize,
}

impl<B: hal::Backend> Borrow<RefCount> for BindGroup<B> {
    fn borrow(&self) -> &RefCount {
        self.life_guard.ref_count.as_ref().unwrap()
    }
}

impl<B: hal::Backend> Borrow<()> for BindGroup<B> {
    fn borrow(&self) -> &() {
        &DUMMY_SELECTOR
    }
}
