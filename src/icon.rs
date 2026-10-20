//! Material Symbols Rounded icons. `scripts/build-fonts.py` reads the
//! table below to subset the icon fonts, so every icon an app using this
//! crate draws must be listed here, with its Material Symbols name.

use iced::widget::{Text, text};
use iced::{Font, Pixels};

use super::font;

macro_rules! icons {
    ($($variant:ident = $codepoint:literal, $name:literal;)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Icon {
            $($variant,)*
        }

        impl Icon {
            pub fn codepoint(self) -> char {
                match self {
                    $(Icon::$variant => char::from_u32($codepoint).unwrap_or('?'),)*
                }
            }
        }
    };
}

icons! {
    AccountTree = 0xe97a, "account_tree";
    Add = 0xe145, "add";
    Album = 0xe019, "album";
    ArrowBack = 0xe5c4, "arrow_back";
    ArrowDropDown = 0xe5c5, "arrow_drop_down";
    ArrowForward = 0xe5c8, "arrow_forward";
    ArrowRightAlt = 0xe941, "arrow_right_alt";
    ArrowSelector = 0xf82f, "arrow_selector_tool";
    ArrowUpward = 0xe5d8, "arrow_upward";
    AspectRatio = 0xe85b, "aspect_ratio";
    AudioFile = 0xeb82, "audio_file";
    AutoStories = 0xe666, "auto_stories";
    Bookmark = 0xe8e7, "bookmark";
    BookmarkAdd = 0xe598, "bookmark_add";
    Bookmarks = 0xe98b, "bookmarks";
    BorderColor = 0xe22b, "border_color";
    BrokenImage = 0xe3ad, "broken_image";
    Cable = 0xefe6, "cable";
    Cast = 0xe307, "cast";
    CastConnected = 0xe308, "cast_connected";
    ChangeCircle = 0xe2e7, "change_circle";
    ChatBubble = 0xe0cb, "chat_bubble";
    Check = 0xe668, "check";
    CheckCircle = 0xe86c, "check_circle";
    ChevronLeft = 0xe5cb, "chevron_left";
    ChevronRight = 0xe5cc, "chevron_right";
    Circle = 0xef4a, "circle";
    Close = 0xe5cd, "close";
    ClosedCaption = 0xe996, "closed_caption";
    Compress = 0xe94d, "compress";
    Computer = 0xe31e, "computer";
    ContentCopy = 0xe14d, "content_copy";
    ContentCut = 0xe14e, "content_cut";
    ContentPaste = 0xe14f, "content_paste";
    Crop = 0xe3be, "crop";
    CropFree = 0xe3c2, "crop_free";
    DarkMode = 0xe51c, "dark_mode";
    Delete = 0xe92e, "delete";
    DesktopWindows = 0xe30c, "desktop_windows";
    Draft = 0xe66d, "draft";
    Draw = 0xe746, "draw";
    Edit = 0xf097, "edit";
    EditDocument = 0xf88c, "edit_document";
    EditSquare = 0xf88d, "edit_square";
    Encrypted = 0xe593, "encrypted";
    Equalizer = 0xe01d, "equalizer";
    Error = 0xf8b6, "error";
    ExpandLess = 0xe5ce, "expand_less";
    ExpandMore = 0xe5cf, "expand_more";
    FileExport = 0xf3b2, "file_export";
    FileOpen = 0xeaf3, "file_open";
    FilterAlt = 0xef4f, "filter_alt";
    FilterList = 0xe152, "filter_list";
    FitPage = 0xf77a, "fit_page";
    FitScreen = 0xea10, "fit_screen";
    FitWidth = 0xf779, "fit_width";
    Forward10 = 0xe056, "forward_10";
    Fullscreen = 0xe5d0, "fullscreen";
    FullscreenExit = 0xe5d1, "fullscreen_exit";
    Language = 0xea07, "language";
    LibraryMusic = 0xe030, "library_music";
    MicOff = 0xe02b, "mic_off";
    Movie = 0xe404, "movie";
    MovieEdit = 0xf840, "movie_edit";
    MusicNote = 0xe405, "music_note";
    OpenInNew = 0xe89e, "open_in_new";
    Pause = 0xe034, "pause";
    PictureInPicture = 0xe911, "picture_in_picture_alt";
    PlayArrow = 0xe037, "play_arrow";
    PlaylistPlay = 0xe05f, "playlist_play";
    QueueMusic = 0xe03d, "queue_music";
    Record = 0xe061, "fiber_manual_record";
    Repeat = 0xe040, "repeat";
    RepeatOne = 0xe041, "repeat_one";
    Replay = 0xe042, "replay";
    Replay10 = 0xe059, "replay_10";
    ScreenRecord = 0xf679, "screen_record";
    ScreenshotMonitor = 0xec08, "screenshot_monitor";
    Shuffle = 0xe043, "shuffle";
    SkipNext = 0xe044, "skip_next";
    SkipPrevious = 0xe045, "skip_previous";
    SlowMotionVideo = 0xe068, "slow_motion_video";
    Speed = 0xe9e4, "speed";
    StopCircle = 0xef71, "stop_circle";
    Subtitles = 0xe048, "subtitles";
    SubtitlesOff = 0xef72, "subtitles_off";
    VideoFile = 0xeb87, "video_file";
    Videocam = 0xe04b, "videocam";
    VideocamOff = 0xe04c, "videocam_off";
    VolumeDown = 0xe04d, "volume_down";
    VolumeMute = 0xe04e, "volume_mute";
    VolumeOff = 0xe04f, "volume_off";
    VolumeUp = 0xe050, "volume_up";
    WidthNormal = 0xf8f6, "width_normal";
    Flare = 0xe3e4, "flare";
    Flip = 0xe3e8, "flip";
    FlipVertical = 0xe005, "flip rotated a quarter turn";
    FolderOpen = 0xe2c8, "folder_open";
    FormatAlignCenter = 0xe234, "format_align_center";
    FormatAlignLeft = 0xe236, "format_align_left";
    FormatAlignRight = 0xe237, "format_align_right";
    FormatBold = 0xe238, "format_bold";
    FormatColorFill = 0xe23a, "format_color_fill";
    FormatColorText = 0xe23c, "format_color_text";
    FormatItalic = 0xe23f, "format_italic";
    FormatStrikethrough = 0xe246, "format_strikethrough";
    FormatUnderlined = 0xe249, "format_underlined";
    Gesture = 0xe155, "gesture";
    GraphicEq = 0xe1b8, "graphic_eq";
    GridOn = 0xe3ec, "grid_on";
    GridView = 0xe9b0, "grid_view";
    Hexagon = 0xeb39, "hexagon";
    HighlightAlt = 0xef52, "highlight_alt";
    History = 0xe8b3, "history";
    HorizontalRule = 0xf108, "horizontal_rule";
    Hub = 0xe9f4, "hub";
    Image = 0xe3f4, "image";
    Info = 0xe88e, "info";
    InkHighlighter = 0xe6d1, "ink_highlighter";
    Input = 0xe890, "input";
    InputCircleLeft = 0xe006, "input_circle rotated a quarter turn";
    KeyboardArrowDown = 0xe313, "keyboard_arrow_down";
    KeyboardArrowUp = 0xe316, "keyboard_arrow_up";
    Lan = 0xeb2f, "lan";
    LeftPanelClose = 0xf717, "left_panel_close";
    LeftPanelOpen = 0xf716, "left_panel_open";
    LightMode = 0xe518, "light_mode";
    Lightbulb = 0xe90f, "lightbulb";
    LineStyle = 0xe919, "line_style";
    LineWeight = 0xe91a, "line_weight";
    Link = 0xe250, "link";
    LinkOff = 0xe16f, "link_off";
    LocationOff = 0xe0c7, "location_off";
    LocationOn = 0xf1db, "location_on";
    Lock = 0xe899, "lock";
    Loupe = 0xe402, "loupe";
    Mic = 0xe31d, "mic";
    MonitorHeart = 0xeaa2, "monitor_heart";
    MoreVert = 0xe5d4, "more_vert";
    NoteAdd = 0xe89c, "note_add";
    OneToOne = 0xefcd, "1x_mobiledata";
    Output = 0xebbe, "output";
    OutputCircleRight = 0xe007, "output_circle rotated a quarter turn";
    Pending = 0xef64, "pending";
    Photo = 0xe412, "photo_camera";
    PhotoLibrary = 0xe413, "photo_library";
    PictureAsPdf = 0xe415, "picture_as_pdf";
    Print = 0xe8ad, "print";
    ProgressActivity = 0xe9d0, "progress_activity";
    RadioButtonUnchecked = 0xe836, "radio_button_unchecked";
    Rectangle = 0xeb54, "rectangle";
    Redo = 0xe15a, "redo";
    Refresh = 0xe5d5, "refresh";
    Remove = 0xe15b, "remove";
    RemoveSelection = 0xe9d5, "remove_selection";
    ResetAll = 0xf053, "restart_alt";
    Resize = 0xf1ce, "open_in_full";
    RightPanelClose = 0xf705, "right_panel_close";
    RightPanelOpen = 0xf704, "right_panel_open";
    RotateLeft = 0xe419, "rotate_left";
    RotateRight = 0xe41a, "rotate_right";
    RoundedCorner = 0xe920, "rounded_corner";
    Router = 0xe328, "router";
    Schedule = 0xefd6, "schedule";
    Search = 0xef7a, "search";
    SelectAll = 0xe162, "select_all";
    Sell = 0xf05b, "sell";
    Settings = 0xe8b8, "settings";
    Shapes = 0xe602, "shapes";
    Signature = 0xf74c, "signature";
    SmartToy = 0xf06c, "smart_toy";
    South = 0xf1e3, "south";
    Speaker = 0xe32d, "speaker";
    Stacks = 0xf500, "stacks";
    Star = 0xf09a, "star";
    StickyNote = 0xf1fc, "sticky_note_2";
    Stop = 0xe047, "stop";
    Stream = 0xe9e9, "stream";
    SyncProblem = 0xe629, "sync_problem";
    TextFields = 0xe262, "text_fields";
    TextFormat = 0xe165, "text_format";
    Timer = 0xe425, "timer";
    Toc = 0xe8de, "toc";
    TopPanelClose = 0xf733, "top_panel_close";
    TopPanelOpen = 0xf732, "top_panel_open";
    Traffic = 0xe565, "traffic";
    Tune = 0xe429, "tune";
    TwoPager = 0xf51f, "two_pager";
    Undo = 0xe166, "undo";
    Upload = 0xf09b, "upload";
    ViewAgenda = 0xe8e9, "view_agenda";
    ViewDay = 0xe8ed, "view_day";
    ViewList = 0xe8ef, "view_list";
    Vignette = 0xe435, "vignette";
    Warning = 0xf083, "warning";
    West = 0xf1e6, "west";
    WifiTethering = 0xe1e2, "wifi_tethering";
    ZoomIn = 0xe8ff, "zoom_in";
    ZoomOut = 0xe900, "zoom_out";
}

/// An outlined icon at `size` pixels.
pub fn icon<'a>(icon: Icon, size: impl Into<Pixels>) -> Text<'a> {
    glyph(icon, size, font::ICONS)
}

/// A filled icon, used for selected states.
pub fn filled<'a>(icon: Icon, size: impl Into<Pixels>) -> Text<'a> {
    glyph(icon, size, font::ICONS_FILLED)
}

fn glyph<'a>(icon: Icon, size: impl Into<Pixels>, font: Font) -> Text<'a> {
    let size = size.into();
    text(icon.codepoint())
        .font(font)
        .size(size)
        .line_height(iced::widget::text::LineHeight::Absolute(size))
        .shaping(iced::widget::text::Shaping::Advanced)
        .center()
}
