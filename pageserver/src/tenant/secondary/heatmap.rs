use std::collections::HashMap;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, TimestampSeconds, serde_as};
use utils::generation::Generation;
use utils::id::TimelineId;

use crate::tenant::remote_timeline_client::index::LayerFileMetadata;
use crate::tenant::storage_layer::LayerName;

#[derive(Serialize, Deserialize)]
pub(crate) struct HeatMapTenant {
    /// Generation of the attached location that uploaded the heatmap: this is not required
    /// for correctness, but acts as a hint to secondary locations in order to detect thrashing
    /// in the unlikely event that two attached locations are both uploading conflicting heatmaps.
    pub(super) generation: Generation,

    pub(super) timelines: Vec<HeatMapTimeline>,

    /// Uploaders provide their own upload period in the heatmap, as a hint to downloaders
    /// of how frequently it is worthwhile to check for updates.
    ///
    /// This is optional for backward compat, and because we sometimes might upload
    /// a heatmap explicitly via API for a tenant that has no periodic upload configured.
    #[serde(default)]
    pub(super) upload_period_ms: Option<u128>,
}

impl HeatMapTenant {
    pub(crate) fn into_timelines_index(self) -> HashMap<TimelineId, HeatMapTimeline> {
        self.timelines
            .into_iter()
            .map(|htl| (htl.timeline_id, htl))
            .collect()
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct HeatMapTimeline {
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) timeline_id: TimelineId,

    layers: Vec<HeatMapLayer>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct HeatMapLayer {
    pub(crate) name: LayerName,
    pub(crate) metadata: LayerFileMetadata,

    #[serde_as(as = "TimestampSeconds<i64>")]
    pub(crate) access_time: SystemTime,

    #[serde(default)]
    pub(crate) cold: bool, // TODO: an actual 'heat' score that would let secondary locations prioritize downloading
                           // the hottest layers, rather than trying to simply mirror whatever layers are on-disk on the primary.
}

impl HeatMapLayer {
    pub(crate) fn new(
        name: LayerName,
        metadata: LayerFileMetadata,
        access_time: SystemTime,
        cold: bool,
    ) -> Self {
        Self {
            name,
            metadata,
            access_time,
            cold,
        }
    }
}

impl HeatMapTimeline {
    pub(crate) fn new(timeline_id: TimelineId, layers: Vec<HeatMapLayer>) -> Self {
        Self {
            timeline_id,
            layers,
        }
    }

    pub(crate) fn into_hot_layers(self) -> impl Iterator<Item = HeatMapLayer> {
        self.layers.into_iter().filter(|l| !l.cold)
    }

    pub(crate) fn hot_layers(&self) -> impl Iterator<Item = &HeatMapLayer> {
        self.layers.iter().filter(|l| !l.cold)
    }

    pub(crate) fn all_layers(&self) -> impl Iterator<Item = &HeatMapLayer> {
        self.layers.iter()
    }
}

pub(crate) struct HeatMapStats {
    pub(crate) bytes: u64,
    pub(crate) layers: usize,
}

impl HeatMapTenant {
    pub(crate) fn get_stats(&self) -> HeatMapStats {
        let mut stats = HeatMapStats {
            bytes: 0,
            layers: 0,
        };
        for timeline in &self.timelines {
            for layer in timeline.hot_layers() {
                stats.layers += 1;
                stats.bytes += layer.metadata.file_size;
            }
        }

        stats
    }

    pub(crate) fn strip_atimes(self) -> Self {
        Self {
            timelines: self
                .timelines
                .into_iter()
                .map(|mut tl| {
                    for layer in &mut tl.layers {
                        layer.access_time = SystemTime::UNIX_EPOCH;
                    }
                    tl
                })
                .collect(),
            generation: self.generation,
            upload_period_ms: self.upload_period_ms,
        }
    }
}
