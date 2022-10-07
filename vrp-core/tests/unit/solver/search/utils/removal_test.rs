use super::*;
use crate::helpers::models::domain::{create_empty_solution_context, create_registry_context};
use crate::helpers::models::problem::{test_driver, test_vehicle_with_id, FleetBuilder, SingleBuilder};
use crate::helpers::models::solution::{create_route_context_with_activities, test_activity_with_job};
use crate::helpers::utils::random::FakeRandom;
use crate::models::common::Dimensions;
use crate::models::problem::{Fleet, Multi};
use std::iter::once;

fn create_route_with_jobs_activities(
    fleet: &Fleet,
    vehicle_idx: usize,
    jobs: usize,
    activities: usize,
) -> RouteContext {
    assert!(jobs > 0);
    assert!(activities >= jobs);

    let vehicle = format!("v{}", vehicle_idx + 1);
    let activities_per_job = activities / jobs;
    let left_overs = activities - activities_per_job * jobs;
    let get_activity = |job_idx: usize| {
        test_activity_with_job(SingleBuilder::default().id(format!("{}", job_idx).as_str()).build_shared())
    };
    // NOTE need to keep multi-jobs somewhere to keep weak reference in sub-jobs alive
    let mut multi_jobs = Vec::new();

    let activities = (0..jobs)
        .flat_map(|job_idx| {
            if activities_per_job > 1 {
                let singles = (0..activities_per_job)
                    .map(|activity_idx| {
                        SingleBuilder::default().id(format!("{}_{}", job_idx, activity_idx).as_str()).build_shared()
                    })
                    .collect::<Vec<_>>();
                let multi = Multi::new_shared(singles, Dimensions::default());
                multi_jobs.push(multi.clone());
                multi.jobs.iter().cloned().map(test_activity_with_job).collect::<Vec<_>>().into_iter()
            } else {
                once(get_activity(job_idx)).collect::<Vec<_>>().into_iter()
            }
        })
        .chain((0..left_overs).map(|idx| get_activity(jobs + idx)))
        .collect();

    let mut route_ctx = create_route_context_with_activities(fleet, vehicle.as_str(), activities);
    route_ctx.state_mut().put_route_state(0, multi_jobs);

    route_ctx
}

fn create_fleet(vehicles: usize) -> Fleet {
    FleetBuilder::default()
        .add_driver(test_driver())
        .add_vehicles((0..vehicles).map(|idx| test_vehicle_with_id(format!("v{}", idx + 1).as_str())).collect())
        .build()
}

fn create_solution_ctx(fleet: &Fleet, routes: Vec<(usize, usize)>) -> SolutionContext {
    let mut registry = create_registry_context(fleet);
    let routes = routes
        .into_iter()
        .enumerate()
        .map(|(idx, (jobs, activities))| create_route_with_jobs_activities(fleet, idx, jobs, activities))
        .collect::<Vec<_>>();
    routes.iter().for_each(|route_ctx| {
        registry.use_route(route_ctx);
    });

    SolutionContext { routes, registry, ..create_empty_solution_context() }
}

fn get_job_from_solution_ctx(solution_ctx: &SolutionContext, route_idx: usize, activity_idx: usize) -> Job {
    solution_ctx.routes.get(route_idx).unwrap().route.tour.get(activity_idx).unwrap().retrieve_job().unwrap()
}

parameterized_test! {can_try_remove_job_with_job_limit, (jobs_activities, route_activity_idx, limits, expected_removed_activities), {
    can_try_remove_job_with_job_limit_impl(jobs_activities, route_activity_idx, limits, expected_removed_activities);
}}

can_try_remove_job_with_job_limit! {
    case_01: ((10, 10), (0, 1), (10, 20, 2), 1),
    case_02: ((10, 20), (0, 1), (10, 20, 2), 2),
    case_03: ((10, 30), (0, 1), (10, 20, 2), 3),
    case_04: ((10, 10), (0, 1), (0, 1, 1), 0),
    case_05: ((10, 10), (0, 1), (1, 0, 0), 0),
}

fn can_try_remove_job_with_job_limit_impl(
    jobs_activities: (usize, usize),
    route_activity_idx: (usize, usize),
    limits: (usize, usize, usize),
    expected_removed_activities: usize,
) {
    let (jobs, activities) = jobs_activities;
    let (route_idx, activity_idx) = route_activity_idx;
    let (max_ruined_jobs, max_ruined_activities, max_affected_routes) = limits;
    let limits = RuinLimitsEx { max_ruined_jobs, max_ruined_activities, max_affected_routes };

    let fleet = create_fleet(1);
    let mut solution_ctx = create_solution_ctx(&fleet, vec![(jobs, activities)]);
    let mut route_ctx = solution_ctx.routes.get_mut(0).cloned().unwrap();
    let job = get_job_from_solution_ctx(&solution_ctx, route_idx, activity_idx);
    let mut removal = JobRemoval::new(&limits);

    let result = removal.try_remove_job(&mut solution_ctx, &mut route_ctx, &job);

    if expected_removed_activities > 0 {
        assert!(result);
        assert_eq!(solution_ctx.required.len(), 1);
        assert!(solution_ctx.required[0] == job);
        assert_eq!(solution_ctx.routes[0].route.tour.job_activity_count(), activities - expected_removed_activities);
        assert_eq!(removal.jobs_left, (max_ruined_jobs - 1) as i32);
        assert_eq!(removal.activities_left, (max_ruined_activities - expected_removed_activities) as i32);
    } else {
        assert!(!result);
        assert!(solution_ctx.required.is_empty());
        assert_eq!(solution_ctx.routes[0].route.tour.job_activity_count(), activities);
        assert_eq!(removal.jobs_left, max_ruined_jobs as i32);
        assert_eq!(removal.activities_left, max_ruined_activities as i32);
    }
}

parameterized_test! {can_try_remove_route_with_limit, (jobs_activities, limits, is_random_hit, expected_affected), {
    can_try_remove_route_with_limit_impl(jobs_activities, limits, is_random_hit, expected_affected);
}}

can_try_remove_route_with_limit! {
    case_01_one_route_left: ((10, 10), (10, 10, 1), false, (10, 10, 1, 0)),
    case_02_no_routes_left: ((10, 10), (10, 10, 0), false, (0, 0, 0, 1)),
    case_03_partial_remove: ((10, 10), (9, 9, 1), false, (9, 9, 1, 1)),
    case_04_fully_remove_by_jobs: ((10, 10), (10, 9, 1), false, (10, 10, 1, 0)),
    case_05_fully_remove_by_hit: ((10, 10), (9, 9, 1), true, (10, 10, 1, 0)),
}

fn can_try_remove_route_with_limit_impl(
    jobs_activities: (usize, usize),
    limits: (usize, usize, usize),
    is_random_hit: bool,
    expected_affected: (usize, usize, usize, usize),
) {
    let (jobs, activities) = jobs_activities;
    let (max_ruined_jobs, max_ruined_activities, max_affected_routes) = limits;

    let limits = RuinLimitsEx { max_ruined_jobs, max_ruined_activities, max_affected_routes };
    let fleet = create_fleet(1);
    let mut solution_ctx = create_solution_ctx(&fleet, vec![(jobs, activities)]);
    let mut route_ctx = solution_ctx.routes.get_mut(0).cloned().unwrap();
    let random = FakeRandom::new(vec![], vec![if is_random_hit { 0. } else { 10. }]);
    let mut removal = JobRemoval::new(&limits);

    let result = removal.try_remove_route(&mut solution_ctx, &mut route_ctx, &random);

    let (expected_affected_activities, expected_affected_jobs, expected_affected_routes, expected_result_routes) =
        expected_affected;
    if expected_affected_routes == 1 {
        assert!(result);
        assert_eq!(removal.jobs_left, (max_ruined_jobs as i32 - expected_affected_jobs as i32).max(0));
        assert_eq!(
            removal.activities_left,
            (max_ruined_activities as i32 - expected_affected_activities as i32).max(0)
        );
        assert_eq!(removal.routes_left, (max_affected_routes - expected_affected_routes) as i32);
        assert_eq!(solution_ctx.required.len(), expected_affected_jobs);
        assert_eq!(solution_ctx.routes.len(), expected_result_routes);
        assert_eq!(solution_ctx.registry.next().count(), 1 - expected_result_routes);
    } else {
        assert!(!result);
        assert!(solution_ctx.required.is_empty());
        assert_eq!(solution_ctx.routes.len(), 1);
        assert_eq!(solution_ctx.routes[0].route.tour.jobs().count(), jobs);
        assert_eq!(solution_ctx.registry.next().count(), 0);
    }
}
