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

pub const QUERY_LIBRARY: &str = "
query ($userId: Int, $type: MediaType) {
  MediaListCollection(userId: $userId, type: $type) {
    lists {
      name
      status
      isCustomList
      entries {
        id
        mediaId
        status
        progress
        progressVolumes
        media {
          title {
            romaji
            english
            native
            userPreferred
          }
        }
      }
    }
  }
}
";

pub const UPDATE_MEDIA: &str = "
mutation(
  $id: Int,
  $mediaId: Int,
  $progress: Int,
) {
  SaveMediaListEntry(id: $id, mediaId: $mediaId, progress: $progress) {
    id
    progress
  }
}
";
