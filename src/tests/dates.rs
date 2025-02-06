pub fn assert_within_minute(
    left: chrono::DateTime<chrono::Utc>,
    right: chrono::DateTime<chrono::Utc>,
) {
    assert!(
        eq_within_minute(left, right),
        "left: {:?}, right: {:?}",
        left,
        right
    );
}

pub fn eq_within_minute(
    left: chrono::DateTime<chrono::Utc>,
    right: chrono::DateTime<chrono::Utc>,
) -> bool {
    left.signed_duration_since(right) <= chrono::Duration::minutes(1)
        && right.signed_duration_since(left) <= chrono::Duration::minutes(1)
}

pub fn assert_within_second(
    left: chrono::DateTime<chrono::Utc>,
    right: chrono::DateTime<chrono::Utc>,
) {
    assert!(
        eq_within_second(left, right),
        "left: {:?}, right: {:?}",
        left,
        right
    );
}

pub fn eq_within_second(
    left: chrono::DateTime<chrono::Utc>,
    right: chrono::DateTime<chrono::Utc>,
) -> bool {
    left.signed_duration_since(right) <= chrono::Duration::seconds(1)
        && right.signed_duration_since(left) <= chrono::Duration::seconds(1)
}

pub fn assert_now_within_second(left: chrono::DateTime<chrono::Utc>) {
    assert!(is_now_within_second(left), "left: {:?}", left);
}

pub fn is_now_within_minute(left: chrono::DateTime<chrono::Utc>) -> bool {
    left.signed_duration_since(chrono::Utc::now()) <= chrono::Duration::minutes(1)
        && chrono::Utc::now().signed_duration_since(left) <= chrono::Duration::minutes(1)
}

pub fn assert_is_now_within_second(left: chrono::DateTime<chrono::Utc>) {
    assert!(is_now_within_second(left), "left: {:?}", left);
}

pub fn is_now_within_second(left: chrono::DateTime<chrono::Utc>) -> bool {
    left.signed_duration_since(chrono::Utc::now()) <= chrono::Duration::seconds(1)
        && chrono::Utc::now().signed_duration_since(left) <= chrono::Duration::seconds(1)
}
