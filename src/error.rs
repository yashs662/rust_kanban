use crate::app::app_helper::NavigationDirection;

#[derive(Debug)]
pub enum NavigationError {
    NoBoardsFound(NavigationDirection),
    CurrentBoardNotFound(NavigationDirection),
    CurrentBoardHasNoCards(NavigationDirection),
    AlreadyAtFirstBoard,
    AlreadyAtLastBoard,
    AlreadyAtFirstCard,
    AlreadyAtLastCard,
    SomethingWentWrong(NavigationDirection),
}

impl std::fmt::Display for NavigationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NavigationError::NoBoardsFound(nav_direction) => {
                write!(f, "Cannot go {:?}: No boards found", nav_direction)
            }
            NavigationError::CurrentBoardNotFound(nav_direction) => {
                write!(f, "Cannot go {:?}: Current board not found", nav_direction)
            }
            NavigationError::CurrentBoardHasNoCards(nav_direction) => write!(
                f,
                "Cannot go {:?}: Current board has no cards",
                nav_direction
            ),
            NavigationError::AlreadyAtFirstBoard => {
                write!(
                    f,
                    "Cannot go {:?}: Already at first board",
                    NavigationDirection::Left
                )
            }
            NavigationError::AlreadyAtLastBoard => {
                write!(
                    f,
                    "Cannot go {:?}: Already at last board",
                    NavigationDirection::Right
                )
            }
            NavigationError::AlreadyAtFirstCard => {
                write!(
                    f,
                    "Cannot go {:?}: Already at first card",
                    NavigationDirection::Up
                )
            }
            NavigationError::AlreadyAtLastCard => {
                write!(
                    f,
                    "Cannot go {:?}: Already at last card",
                    NavigationDirection::Down
                )
            }
            NavigationError::SomethingWentWrong(nav_direction) => {
                write!(f, "Cannot go {:?}: Something went wrong", nav_direction)
            }
        }
    }
}

impl std::error::Error for NavigationError {}
