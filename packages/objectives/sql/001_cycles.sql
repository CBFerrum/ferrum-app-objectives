-- Objectives 0.8.0 (package 0.2.0): objectives belong to a cycle, may be private, and may be drafts.
-- A workspace already there gets the Default cycle, active, holding every objective it has; nothing
-- is private and nothing a draft until someone says so. Its roles keep the rules their package gave
-- them until the app is updated, so they get the new ones here, and the rules that read every
-- objective, key result and check-in (only those still as the package gave them) lose that reach
-- to the rules that leave out private ones.
select set_config('ferrum.system', 'on', true);
insert into e_okr_cycle (tenant_id, id, cycle_name, is_default, status)
select i.tenant_id, gen_random_uuid(), 'Default', true, 'Active'
from installed_package i
where i.name = 'objectives' and not exists (select 1 from e_okr_cycle c where c.tenant_id = i.tenant_id);
update e_objective o set cycle = c.id from e_okr_cycle c
where o.cycle is null and c.tenant_id = o.tenant_id and c.is_default;
delete from role_permission p using role r
where r.tenant_id = p.tenant_id and r.name = p.role and r.package = 'objectives'
  and not p.if_owner and p.rule_key = '' and p.match_rule is null and p.via is null and p.entity in ('Objective', 'KeyResult', 'CheckIn')
  and p.role in ('Objectives Owner', 'Objectives Viewer') and not p.can_delete and not p.can_submit and not p.can_cancel and not p.can_import;
insert into role_permission (tenant_id, role, entity, can_read, can_write, can_create, can_delete, can_submit, can_cancel, can_export, can_import,
                             if_owner, read_groups, write_groups, match_rule, rule_key, via, actions)
select r.tenant_id, r.name, x.entity, true, x.w, x.w, false, false, false, x.e, false, x.via is not null, '{}', '{}', x.m, x.k, x.via, '{}'
from role r
join (values
  ('Objectives Owner', 'Objective', true, true, '{"private": false}'::jsonb, '[["private",["false"]]]', null),
  ('Objectives Owner', 'Objective', true, true, null, 'via:owner', 'owner'),
  ('Objectives Owner', 'Objective', false, false, null, 'via:people.person', 'people.person'),
  ('Objectives Owner', 'KeyResult', true, true, '{"objective_private": false}', '[["objective_private",["false"]]]', null),
  ('Objectives Owner', 'KeyResult', true, true, null, 'via:objective.owner', 'objective.owner'),
  ('Objectives Owner', 'KeyResult', false, false, null, 'via:objective.people.person', 'objective.people.person'),
  ('Objectives Owner', 'CheckIn', true, false, '{"objective_private": false}', '[["objective_private",["false"]]]', null),
  ('Objectives Owner', 'CheckIn', true, false, null, 'via:key_result.objective.owner', 'key_result.objective.owner'),
  ('Objectives Owner', 'CheckIn', false, false, null, 'via:key_result.objective.people.person', 'key_result.objective.people.person'),
  ('Objectives Owner', 'OkrCycle', true, false, null, '', null),
  ('Objectives Viewer', 'Objective', false, false, '{"private": false}', '[["private",["false"]]]', null),
  ('Objectives Viewer', 'Objective', false, false, null, 'via:owner', 'owner'),
  ('Objectives Viewer', 'Objective', false, false, null, 'via:people.person', 'people.person'),
  ('Objectives Viewer', 'KeyResult', false, false, '{"objective_private": false}', '[["objective_private",["false"]]]', null),
  ('Objectives Viewer', 'KeyResult', false, false, null, 'via:objective.owner', 'objective.owner'),
  ('Objectives Viewer', 'KeyResult', false, false, null, 'via:objective.people.person', 'objective.people.person'),
  ('Objectives Viewer', 'CheckIn', false, false, '{"objective_private": false}', '[["objective_private",["false"]]]', null),
  ('Objectives Viewer', 'CheckIn', false, false, null, 'via:key_result.objective.owner', 'key_result.objective.owner'),
  ('Objectives Viewer', 'CheckIn', false, false, null, 'via:key_result.objective.people.person', 'key_result.objective.people.person'),
  ('Objectives Viewer', 'OkrCycle', false, false, null, '', null)
) as x(role, entity, w, e, m, k, via) on x.role = r.name
where r.package = 'objectives'
on conflict do nothing;
