# ubu-block
UBU Blockchain

## Running
```sql
ubu-block query -q "Select 
  c.title as candidate, 
  SUM(votes) as votes, 
  ward_name as ward, 
  constituency_name as constituency, 
  county_name as county,
  parties.title as party
from 
  results 
  INNER JOIN stations ON stations.id = results.station_id 
  INNER JOIN candidates c ON c.id = results.candidate_id 
  INNER JOIN wards on stations.ward_code = wards.ward_code 
  INNER JOIN parties ON parties.id = c.party_id 
  INNER JOIN constituencies ON wards.constituency_code = constituencies.constituency_code 
  INNER JOIN counties ON constituencies.county_code = counties.county_code 
WHERE 
  position_type = 'Mp'  and constituency = 'Juja'
GROUP BY 
  c.id ORDER by votes DESC ;"
 ```
 
 You should get something like
 ```
 +--------+--------------+----------+-----------+-------+-------+
| county | constituency |   ward   | candidate | party | votes |
+--------+--------------+----------+-----------+-------+-------+
| Kiambu |     Juja     | Kalimoni |   Mwas    |  ODM  |  738  |
+--------+--------------+----------+-----------+-------+-------+
| Kiambu |     Juja     | Kalimoni |   Omosh   |  PNU  |  228  |
+--------+--------------+----------+-----------+-------+-------+
```
