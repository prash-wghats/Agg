use agg::path_storage::PathStorage;

pub fn make_arrows(ps: &mut PathStorage) {
    ps.remove_all();

    ps.move_to(1330.599999999999909, 1282.399999999999864);
    ps.line_to(1377.400000000000091, 1282.399999999999864);
    ps.line_to(1361.799999999999955, 1298.000000000000000);
    ps.line_to(1393.000000000000000, 1313.599999999999909);
    ps.line_to(1361.799999999999955, 1344.799999999999955);
    ps.line_to(1346.200000000000045, 1313.599999999999909);
    ps.line_to(1330.599999999999909, 1329.200000000000045);
    ps.close_polygon(0);

    ps.move_to(1330.599999999999909, 1266.799999999999955);
    ps.line_to(1377.400000000000091, 1266.799999999999955);
    ps.line_to(1361.799999999999955, 1251.200000000000045);
    ps.line_to(1393.000000000000000, 1235.599999999999909);
    ps.line_to(1361.799999999999955, 1204.399999999999864);
    ps.line_to(1346.200000000000045, 1235.599999999999909);
    ps.line_to(1330.599999999999909, 1220.000000000000000);
    ps.close_polygon(0);

    ps.move_to(1315.000000000000000, 1282.399999999999864);
    ps.line_to(1315.000000000000000, 1329.200000000000045);
    ps.line_to(1299.400000000000091, 1313.599999999999909);
    ps.line_to(1283.799999999999955, 1344.799999999999955);
    ps.line_to(1252.599999999999909, 1313.599999999999909);
    ps.line_to(1283.799999999999955, 1298.000000000000000);
    ps.line_to(1268.200000000000045, 1282.399999999999864);
    ps.close_polygon(0);

    ps.move_to(1268.200000000000045, 1266.799999999999955);
    ps.line_to(1315.000000000000000, 1266.799999999999955);
    ps.line_to(1315.000000000000000, 1220.000000000000000);
    ps.line_to(1299.400000000000091, 1235.599999999999909);
    ps.line_to(1283.799999999999955, 1204.399999999999864);
    ps.line_to(1252.599999999999909, 1235.599999999999909);
    ps.line_to(1283.799999999999955, 1251.200000000000045);
    ps.close_polygon(0);
}