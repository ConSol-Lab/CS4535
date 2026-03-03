use std::fmt::Debug;

use crate::conflict_resolving::ConflictAnalysisContext;
#[cfg(doc)]
use crate::create_statistics_struct;
use crate::predicates::Predicate;
use crate::statistics::StatisticLogger;

/// A trait for the behaviour of nogood minimisation approaches.
///
/// See [`NogoodMinimiser::minimise`] for more information.
pub trait NogoodMinimiser: Debug {
    /// Takes as input a nogood represented by a [`Vec`] of [`Predicate`]s and minimises the
    /// nogood by removing redundant [`Predicate`]s.
    fn minimise(&mut self, context: &mut ConflictAnalysisContext, nogood: &mut Vec<Predicate>);

    /// Logs statistics of the nogood minimiser using the provided [`StatisticLogger`].
    ///
    /// It is recommended to create a struct through the [`create_statistics_struct!`] macro!
    fn log_statistics(&self, _statistic_logger: StatisticLogger) {}
}
