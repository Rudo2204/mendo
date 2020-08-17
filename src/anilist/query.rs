pub const QUERY_USER: &str = "
query {
  Viewer {
    id
    name
    siteUrl
    updatedAt
  }
}
";

pub const SEARCH_MEDIA: &str = "
query ($search: String, $type: MediaType, $status_not: MediaStatus) {
    Media(search: $search, type: $type, status_not: $status_not) {
        id
        status
        title {
            romaji
            english
            native
        }
        synonyms
        chapters
        volumes
    }
}
";

pub const QUERY_MEDIA_LIST: &str = "
query ($userId: Int, $mediaId: Int, $type: MediaType, $status_not: MediaListStatus) {
    MediaList(userId: $userId, mediaId: $mediaId, type: $type, status_not: $status_not) {
        id
        status
        progress
    }
}
";

pub const UPDATE_MEDIA: &str = "
mutation(
  $id: Int,
  $mediaId: Int,
  $status: MediaListStatus
  $progress: Int,
) {
  SaveMediaListEntry(
      id: $id,
      mediaId: $mediaId,
      status: $status,
      progress: $progress) {
    id
    mediaId
    status
    progress
  }
}
";
