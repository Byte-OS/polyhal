# For Async Runtime Design

> This is a design for async runtime.

Design a TrapFrom::into_user() Then it will save the registers and use the pointer.

Make a struct ThreadToken to pointer the original sp that contains the registers.

Add a ThreadToken::restore() method to the original position, and it can return the custom value to the original position.
