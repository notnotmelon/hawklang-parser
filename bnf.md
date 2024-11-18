Rule 01: PROGRAM -> program DECL_SEC begin STMT_SEC end; | program begin STMT_SEC end;

Rule 02: DECL_SEC -> DECL | DECL DECL_SEC

Rule 03: DECL -> ID_LIST : TYPE ;

Rule 04: ID_LIST -> ID | ID , ID_LIST

Rule 05: ID -> (_ | a | b | ... | z | A | ... | Z) (_ | a | b | ... | z | A | ... | Z | 0 | 1 | ... | 9)*

Rule 06: STMT_SEC -> STMT | STMT STMT_SEC

Rule 07: STMT -> ASSIGN | IFSTMT | WHILESTMT | INPUT | OUTPUT

Rule 08: ASSIGN -> ID := EXPR ;

Rule 09: IFSTMT -> if COMP then STMT_SEC end if ; | if COMP then STMT_SEC else STMT_SEC end if ;

Rule 10: WHILESTMT -> while COMP loop STMT_SEC end loop ;

Rule 11: INPUT -> input ID_LIST;

Rule 12: OUTPUT -> output ID_LIST | output NUM;

Rule 13: EXPR -> FACTOR | FACTOR + EXPR | FACTOR - EXPR

Rule 14: FACTOR -> OPERAND | OPERAND * FACTOR | OPERAND / FACTOR

Rule 15: OPERAND -> NUM | ID | ( EXPR )

Rule 16: NUM -> (0 | 1 | ... | 9)+ [.(0 | 1 | ... | 9)+]

Rule 17: COMP -> ( OPERAND = OPERAND ) | ( OPERAND <> OPERAND ) | ( OPERAND > OPERAND ) | ( OPERAND < OPERAND )

Rule 18: TYPE -> int | float | double