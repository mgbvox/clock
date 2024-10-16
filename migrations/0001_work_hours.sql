create table work_hours (
    id integer primary key autoincrement,
    job_name text not null,
    clock_in datetime not null,
    clock_out datetime,
    message text
);

create unique index id_index on work_hours (id);