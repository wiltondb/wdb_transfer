/*
 * Copyright 2024, WiltonDB Software
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use super::*;

pub fn load_tables_from_db<P: Fn(&str)->()>(progress_fun: &P, conn_config: &TdsConnConfig, dbname: &str) -> Result<Vec<TableWithRowsCount>, TransferError> {
    let runtime = conn_config.create_runtime()?;
    let mut client = conn_config.open_connection_to_db(&runtime, dbname)?;
    runtime.block_on(async {
        progress_fun("Loading tables ...");
        let qr_bbf = tiberius::Query::new("\
                select
                    schema_name(tb.schema_id) as table_schema,
                    tb.name as table_name,
                    case
                        when pc.reltuples is null then cast(-1 as bigint)
                        when pc.reltuples = -1 then cast(0 as bigint)
                        else cast(pc.reltuples as bigint)
                    end as row_count
                from sys.tables as tb
                left join pg_catalog.pg_class pc
                  on pc.relnamespace = tb.schema_id
                  and pc.relname = tb.name
                where
                    pc.relkind in ('r', 'f', 'p')");
        let qs_bbf = qr_bbf.query(&mut client).await;
        let qs = if let Ok(qs) = qs_bbf {
            qs
        } else {
            let qr_mssql = tiberius::Query::new("\
                    select
                        schema_name(tb.schema_id) as table_schema,
                        tb.name as table_name,
                        case
                            when st.row_count is null then cast(-1 as bigint)
                            else st.row_count
                        end as row_count
                    from sys.tables as tb
                    left join sys.dm_db_partition_stats as st
                        on tb.object_id = st.object_id
                    where
                        tb.type_desc = 'USER_TABLE'
                        and st.index_id IN (0, 1)");
            std::mem::drop(qs_bbf);
            let qs_mssql = qr_mssql.query(&mut client).await;
            if let Ok(qs) = qs_mssql {
                qs
            } else {
                let mut qr_generic = tiberius::Query::new("\
                        select
                            table_schema,
                            table_name,
                            cast(-1 as bigint) as row_count
                        from information_schema.tables
                        where table_type = 'BASE TABLE'
                        and table_catalog = @P1");
                qr_generic.bind(dbname);
                std::mem::drop(qs_mssql);
                qr_generic.query(&mut client).await?
            }
        };
        let rows = qs.into_first_result().await?;
        let mut tables = Vec::new();
        let msg = "Tables select error";
        for row in rows.iter() {
            let schema: &str = row.get(0).ok_or(TransferError::from_str(msg))?;
            let table: &str = row.get(1).ok_or(TransferError::from_str(msg))?;
            let count: i64 = row.get(2).ok_or(TransferError::from_str(msg))?;
            progress_fun(&format!("{}.{} {} rows", schema, table, count));
            tables.push(TableWithRowsCount::new(schema, table, count));
        }
        Ok(tables)
    })
}

