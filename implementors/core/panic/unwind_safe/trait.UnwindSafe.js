(function() {var implementors = {};
implementors["storage_api"] = [{"text":"impl&lt;A&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a> for <a class=\"struct\" href=\"storage_api/struct.AllocStorage.html\" title=\"struct storage_api::AllocStorage\">AllocStorage</a>&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a>,&nbsp;</span>","synthetic":true,"types":["storage_api::alloc::AllocStorage"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a> for <a class=\"struct\" href=\"storage_api/struct.AllocHandle.html\" title=\"struct storage_api::AllocHandle\">AllocHandle</a>","synthetic":true,"types":["storage_api::alloc::AllocHandle"]},{"text":"impl&lt;'a, DataStore&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a> for <a class=\"struct\" href=\"storage_api/struct.BorrowedStorage.html\" title=\"struct storage_api::BorrowedStorage\">BorrowedStorage</a>&lt;'a, DataStore&gt;","synthetic":true,"types":["storage_api::borrowed::BorrowedStorage"]},{"text":"impl&lt;DataStore&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a> for <a class=\"struct\" href=\"storage_api/struct.InlineStorage.html\" title=\"struct storage_api::InlineStorage\">InlineStorage</a>&lt;DataStore&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;DataStore: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a>,&nbsp;</span>","synthetic":true,"types":["storage_api::inline::InlineStorage"]},{"text":"impl&lt;T:&nbsp;?<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a> for <a class=\"struct\" href=\"storage_api/struct.RawBox.html\" title=\"struct storage_api::RawBox\">RawBox</a>&lt;T, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as <a class=\"trait\" href=\"storage_api/trait.Storage.html\" title=\"trait storage_api::Storage\">Storage</a>&gt;::<a class=\"associatedtype\" href=\"storage_api/trait.Storage.html#associatedtype.Handle\" title=\"type storage_api::Storage::Handle\">Handle</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T as <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ptr/metadata/trait.Pointee.html\" title=\"trait core::ptr::metadata::Pointee\">Pointee</a>&gt;::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/nightly/core/ptr/metadata/trait.Pointee.html#associatedtype.Metadata\" title=\"type core::ptr::metadata::Pointee::Metadata\">Metadata</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a>,&nbsp;</span>","synthetic":true,"types":["storage_api::raw_box::RawBox"]},{"text":"impl&lt;T, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a> for <a class=\"struct\" href=\"storage_api/struct.RawVec.html\" title=\"struct storage_api::RawVec\">RawVec</a>&lt;T, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as <a class=\"trait\" href=\"storage_api/trait.Storage.html\" title=\"trait storage_api::Storage\">Storage</a>&gt;::<a class=\"associatedtype\" href=\"storage_api/trait.Storage.html#associatedtype.Handle\" title=\"type storage_api::Storage::Handle\">Handle</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a>,&nbsp;</span>","synthetic":true,"types":["storage_api::raw_vec::RawVec"]},{"text":"impl&lt;DataStore, A&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a> for <a class=\"struct\" href=\"storage_api/struct.SmallStorage.html\" title=\"struct storage_api::SmallStorage\">SmallStorage</a>&lt;DataStore, A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;DataStore: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html\" title=\"trait core::panic::unwind_safe::UnwindSafe\">UnwindSafe</a>,&nbsp;</span>","synthetic":true,"types":["storage_api::small::SmallStorage"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()