Rule 01: PROGRAM à program DECL_SEC begin STMT_SEC end; |
program begin STMT_SEC end;

Rule 02: DECL_SEC à DECL | DECL DECL_SEC

Rule 03: DECL à ID_LIST : TYPE ;

Rule 04: ID_LIST à ID | ID , ID_LIST

Rule 05: ID à (_ | a | b | ... | z | A | ... | Z) (_ | a | b | ... | z | A | ... | Z | 0 | 1 | ... | 9)*

Rule 06: STMT_SEC à STMT | STMT STMT_SEC

Rule 07: STMT à ASSIGN | IFSTMT | WHILESTMT | INPUT | OUTPUT

Rule 08: ASSIGN à ID := EXPR ;

Rule 09: IFSTMT à if COMP then STMT_SEC end if ; |
if COMP then STMT_SEC else STMT_SEC end if ;

Rule 10: WHILESTMT à while COMP loop STMT_SEC end loop ;

Rule 11: INPUT à input ID_LIST;

Rule 12: OUTPUT à output ID_LIST | output NUM;

Rule 13: EXPR à FACTOR | FACTOR + EXPR | FACTOR - EXPR

Rule 14: FACTOR à OPERAND | OPERAND * FACTOR | OPERAND / FACTOR

Rule 15: OPERAND à NUM | ID | ( EXPR )

Rule 16: NUM à (0 | 1 | ... | 9)+ [.(0 | 1 | ... | 9)+]

Rule 17: COMP à ( OPERAND = OPERAND ) | ( OPERAND <> OPERAND ) |
( OPERAND > OPERAND ) | ( OPERAND < OPERAND )

Rule 18: TYPE à int | float | double