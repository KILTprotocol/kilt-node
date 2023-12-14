// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at info@botlabs.org

#![cfg_attr(not(feature = "std"), no_std)]

pub mod post;
pub mod traits;

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		traits::EnsureOrigin,
		BoundedVec,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Hash;
	use sp_std::fmt::Debug;

	use crate::{
		post::{Comment, Post},
		traits::GetUsername,
	};

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	pub type BoundedTextOf<T> = BoundedVec<u8, <T as Config>::MaxTextLength>;
	pub type PostOf<T> = Post<<T as frame_system::Config>::Hash, BoundedTextOf<T>, <T as Config>::Username>;
	pub type CommentOf<T> = Comment<<T as frame_system::Config>::Hash, BoundedTextOf<T>, <T as Config>::Username>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type MaxTextLength: Get<u32>;
		type OriginCheck: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin, Success = Self::OriginSuccess>;
		type OriginSuccess: GetUsername<Username = Self::Username>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Username: Encode + Decode + TypeInfo + MaxEncodedLen + Clone + PartialEq + Debug + Default;
	}

	#[pallet::storage]
	#[pallet::getter(fn posts)]
	pub type Posts<T> = StorageMap<_, Twox64Concat, <T as frame_system::Config>::Hash, PostOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn comments)]
	pub type Comments<T> = StorageMap<_, Twox64Concat, <T as frame_system::Config>::Hash, CommentOf<T>>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewPost {
			post_id: T::Hash,
			author: T::Username,
		},
		NewComment {
			resource_id: T::Hash,
			comment_id: T::Hash,
			author: T::Username,
		},
		NewLike {
			resource_id: T::Hash,
			liker: T::Username,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(1_000)]
		pub fn post(origin: OriginFor<T>, text: BoundedTextOf<T>) -> DispatchResult {
			let success_origin = T::OriginCheck::ensure_origin(origin)?;
			let author = success_origin.username().map_err(DispatchError::Other)?;
			let post_id = T::Hashing::hash(
				(&frame_system::Pallet::<T>::block_number(), &author, &text)
					.encode()
					.as_slice(),
			);
			let post = PostOf::<T>::from_text_and_author(text, author.clone());
			Posts::<T>::insert(post_id, post);
			Self::deposit_event(Event::NewPost { post_id, author });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(1_000)]
		pub fn comment(origin: OriginFor<T>, resource_id: T::Hash, text: BoundedTextOf<T>) -> DispatchResult {
			let success_origin = T::OriginCheck::ensure_origin(origin)?;
			let author = success_origin.username().map_err(DispatchError::Other)?;
			let comment_id = T::Hashing::hash(
				(&frame_system::Pallet::<T>::block_number(), &author, &text)
					.encode()
					.as_slice(),
			);
			Posts::<T>::try_mutate(resource_id, |post| {
				if let Some(post) = post {
					post.comments
						.try_push(comment_id)
						.expect("Failed to add comment to post.");
					Ok(())
				} else {
					Err(())
				}
			})
			.or_else(|_| {
				Comments::<T>::try_mutate(resource_id, |comment| {
					if let Some(comment) = comment {
						comment
							.details
							.comments
							.try_push(comment_id)
							.expect("Failed to add comment to comment.");
						Ok(())
					} else {
						Err(())
					}
				})
			})
			.map_err(|_| DispatchError::Other("No post or comment with provided ID found."))?;
			let comment = CommentOf::<T>::from_post_id_text_and_author(resource_id, text, author.clone());
			Comments::<T>::insert(comment_id, comment);
			Self::deposit_event(Event::NewComment {
				resource_id,
				comment_id,
				author,
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(1_000)]
		pub fn like(origin: OriginFor<T>, resource_id: T::Hash) -> DispatchResult {
			let success_origin = T::OriginCheck::ensure_origin(origin)?;
			let liker = success_origin.username().map_err(DispatchError::Other)?;
			Posts::<T>::try_mutate(resource_id, |post| {
				if let Some(post) = post {
					post.likes.try_push(liker.clone()).expect("Failed to add like to post.");
					Ok(())
				} else {
					Err(())
				}
			})
			.or_else(|_| {
				Comments::<T>::try_mutate(resource_id, |comment| {
					if let Some(comment) = comment {
						comment
							.details
							.likes
							.try_push(liker.clone())
							.expect("Failed to add like to comment.");
						Ok(())
					} else {
						Err(())
					}
				})
			})
			.map_err(|_| DispatchError::Other("No post or comment with provided ID found."))?;
			Self::deposit_event(Event::NewLike { resource_id, liker });
			Ok(())
		}
	}
}
