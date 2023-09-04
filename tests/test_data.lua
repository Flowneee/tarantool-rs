box.cfg {}

-- User for authentication tests
box.schema.user.create('Sisko', {password = 'A-4-7-1'})

-- Table with test data
box.schema.sequence.create('seq_id_ds9_crew',{min=1, start=1})

ds9_crew = box.schema.space.create('ds9_crew')
ds9_crew:format({
      {'id', type = 'unsigned'},
      {'name', type = 'string'},
      {'rank', type = 'string', is_nullable = true},
      {'occupation', type = 'string', is_nullable = true},
})
ds9_crew:create_index('idx_id', {parts = {1, 'unsigned'}, sequence = 'seq_id_ds9_crew'})
ds9_crew:create_index('idx_name', {parts = {2, 'string'}})
ds9_crew:create_index('idx_rank', {unique = false, parts = {3, 'string', is_nullable = true}})


ds9_crew:auto_increment{'Benjamin Sisko', 'Commander', 'Commanding officer'}
ds9_crew:auto_increment{'Kira Nerys', 'Major', 'First officer'}
ds9_crew:auto_increment{'Jadzia Dax', 'Lieutenant Commander', 'Science officer'}
ds9_crew:auto_increment{'Julian Bashir', 'Lieutenant', 'Chief medical officer'}
ds9_crew:auto_increment{'Miles O\'Brien', 'Senior Chief Petty Officer', 'Chief of operations'}
ds9_crew:auto_increment{'Worf', 'Lieutenant Commander', 'Strategic operations officer'}
ds9_crew:auto_increment{'Odo', 'Colonel (unofficial)', 'Chief of security'}

-- Test function
function station_name(old)
   if old then
      return 'Terok Nor'
   else
      return 'Deep Space 9'
   end
end
