
#include "raylib.h"
#include <raygui.h>
#include "raymath.h"

// #define AM_IMPLEMENTATION
// #include "amoeba.h"      

#include <stdio.h>
#include <math.h>
#include <assert.h>

//========================================

bool g_pinsActive = true;

typedef struct Anchor_t {
    Vector2 p;
    Vector2 p_dragstart;
    Vector2 p_orig;
    bool pinned;
} Anchor;

#define MAX_ANCHOR (100)
Anchor g_anchor[MAX_ANCHOR];
int nAnchor = 0;

int add_anchor( int x, int y )
{
    int ndx = nAnchor++;

    Anchor *newanc = g_anchor + ndx;
    newanc->p.x = x;
    newanc->p.y = y;

    newanc->p_orig = newanc->p;

    return ndx;
}

//========================================

typedef struct Wall_t
{
    int a;
    int b;
} Wall;

#define MAX_WALL (50)
int nWall = 0;
Wall g_wall[MAX_WALL];

int add_wall( int a, int b )
{
    int ndx = nWall++;

    Wall *wall = g_wall + ndx;
    wall->a = a;
    wall->b = b;

    return ndx;
}

//========================================
typedef enum {
    ConstraintType_LENGTH,
    ConstraintType_PARALLEL,
    ConstraintType_ANGLE,
} ConsType;

typedef struct Constraint_t
{
    ConsType type;

    int a;
    int b;
    int c;

    int a2;
    int b2;
    
    // Length constraint
    float targetLen;

    float targetAngle; 
} Constraint;

#define MAX_CONSTRAINT (100)
int nConstraint = 0;
Constraint g_constraint[MAX_CONSTRAINT];

int add_constraint_len( int a, int b, float target )
{
    int ndx = nConstraint++;

    Constraint *cons = g_constraint + ndx;
    cons->a = a;
    cons->b = b;
    cons->type = ConstraintType_LENGTH;

    // Target 0, use current length
    if (target <= 0.0) {
        target = Vector2Distance( 
            g_anchor[a].p, 
                g_anchor[b].p );
    }
    cons->targetLen = target;

    return ndx;
}


bool dbgParrActive = true;
int add_constraint_parallel( int a, int b, int a2, int  b2 )
{
    int ndx = nConstraint++;

    Constraint *cons = g_constraint + ndx;
    cons->a = a;
    cons->b = b;
    cons->a2 = a2;
    cons->b2 = b2;
    cons->type = ConstraintType_PARALLEL;    

    return ndx;
}

bool dbgAngleActive = true;
int add_constraint_angle( int a, int b, int c, float targetAngle )
{
    int ndx = nConstraint++;

    Constraint *cons = g_constraint + ndx;
    cons->a = a;
    cons->b = b;
    cons->c = c;
    
    cons->type = ConstraintType_ANGLE;  

    // Target 0, use current angle
    if (targetAngle <= 0.0) {        
        Vector2 ba = Vector2Subtract( g_anchor[ cons->a ].p, g_anchor[ cons->b ].p );
        Vector2 bc = Vector2Subtract( g_anchor[ cons->c ].p, g_anchor[ cons->b ].p );
        ba = Vector2Normalize( ba );
        bc = Vector2Normalize( bc );

        float dot = ba.x*bc.x + ba.y*bc.y;
        float ang = acos( dot );

        targetAngle = ang;
        printf("Target Angle: %f\n", ang * (180.0f/3.1415f));
    }
    cons->targetAngle = targetAngle;

    return ndx;
}


void eval_constrain_len( Constraint cons, float str )
{
    float curr_d = Vector2Distance( g_anchor[cons.a].p, g_anchor[cons.b].p );
    float diff = curr_d - cons.targetLen;

    Vector2 dir = Vector2Subtract( g_anchor[cons.b].p, g_anchor[cons.a].p );
    dir = Vector2Normalize( dir );

    dir = Vector2Scale( dir, str * 0.5f * diff );
    g_anchor[cons.a].p = Vector2Add( g_anchor[cons.a].p, dir );
    g_anchor[cons.b].p = Vector2Subtract( g_anchor[cons.b].p, dir );
    
}

Vector2 Vector2RotateAroundPoint( Vector2 p, Vector2 center, float angrad )
{
    Vector2 p2 = Vector2Subtract( p, center );
    float s = sin( angrad );
    float c = cos( angrad );

    //p2 = (Vector2){ p2.x*c - p2.y*s, p2.x*s + p2.y*c };
    Vector2 pr;
    pr.x = p2.x*c - p2.y*s;
    pr.y = p2.x*s + p2.y*c;

    return Vector2Add( pr, center );
}

void eval_constrain_parallel( Constraint cons, float str )
{

    // DBG
    if (!dbgParrActive) return;

    Anchor *a1 = g_anchor + cons.a;
    Anchor *b1 = g_anchor + cons.b;
    Anchor *a2 = g_anchor + cons.a2;
    Anchor *b2 = g_anchor + cons.b2;

    float ang1 = atan2( b1->p.y - a1->p.y, b1->p.x - a1->p.x );
    float ang2 = atan2( b2->p.y - a2->p.y, b2->p.x - a2->p.x );

    // todo: handle crossing
    float angDiff = ang2 - ang1;
    //printf("angDiff %f\n", angDiff);

    float ang = angDiff * 0.5f * str;
    Vector2 ctr1 = Vector2Scale( Vector2Add( a1->p, b1->p ), 0.5f);
    a1->p = Vector2RotateAroundPoint( a1->p, ctr1, ang );
    b1->p = Vector2RotateAroundPoint( b1->p, ctr1, ang );

    Vector2 ctr2 = Vector2Scale( Vector2Add( a2->p, b2->p ), 0.5f);
    a2->p = Vector2RotateAroundPoint( a2->p, ctr2, -ang );
    b2->p = Vector2RotateAroundPoint( b2->p, ctr2, -ang );

}

void eval_constrain_angle( Constraint cons, float str )
{

    // DBG
    if (!dbgAngleActive) return;

    Anchor *a = g_anchor + cons.a;
    Anchor *b = g_anchor + cons.b;
    Anchor *c = g_anchor + cons.c;    

    Vector2 ba = Vector2Subtract( a->p, b->p );
    Vector2 bc = Vector2Subtract( c->p, b->p );

    ba = Vector2Normalize( ba );
    bc = Vector2Normalize( bc );

    float dot = ba.x*bc.x + ba.y*bc.y;
    float angCurr = acos( dot );
    
    // todo: handle crossing
    float angDiff = angCurr - cons.targetAngle;    

    float ang = angDiff * 0.5f * str;    
    //printf("dot %f AngDiff %f\n", dot, angDiff );
    a->p = Vector2RotateAroundPoint( a->p, b->p, -ang );
    c->p = Vector2RotateAroundPoint( c->p, b->p, ang );
    

}

void draw_constrain_parallel( Constraint cons )
{
    Anchor *a1 = g_anchor + cons.a;
    Anchor *b1 = g_anchor + cons.b;
    Anchor *a2 = g_anchor + cons.a2;
    Anchor *b2 = g_anchor + cons.b2;

    float ang1 = atan2( b1->p.y - a1->p.y, b1->p.x - a1->p.x );
    float ang2 = atan2( b2->p.y - a2->p.y, b2->p.x - a2->p.x );

    // todo: handle crossing
    float angDiff = ang2 - ang1;    


    Vector2 aprev = a1->p;
    Vector2 bprev = b1->p;
    for (int i=0; i < 10; i++) 
    {
        float ang = angDiff * 0.5f * ((float)i / 10.0f);
        Vector2 ctr1 = Vector2Scale( Vector2Add( a1->p, b1->p ), 0.5f);
        Vector2 ar = Vector2RotateAroundPoint( a1->p, ctr1, ang );
        Vector2 br = Vector2RotateAroundPoint( b1->p, ctr1, ang );
        DrawLineEx( aprev, ar, 2.0f, ORANGE );
        DrawLineEx( bprev, br, 2.0f, ORANGE );
        aprev = ar;
        bprev = br;


        // Vector2 ctr2 = Vector2Scale( Vector2Add( a2->p, b2->p ), 0.5f);
        // a2->p = Vector2RotateAroundPoint( a2->p, ctr2, -ang );
        // b2->p = Vector2RotateAroundPoint( b2->p, ctr2, ang );
    }


}


//========================================

int main() {

	int screenWidth = 690;
    int screenHeight = 560;
    SetConfigFlags(FLAG_WINDOW_RESIZABLE);
    InitWindow(screenWidth, screenHeight, "Proto1");
    SetExitKey(0);
    SetTargetFPS(30);

    // Main game loop
    bool exitWindow = false;
    bool showMessageBox = false;

    // // first, create a solver:
    // am_Solver *S = am_newsolver(NULL, NULL);

    // // create some variable:
    // am_Var *l = am_newvariable(S);
    // am_Var *m = am_newvariable(S);
    // am_Var *r = am_newvariable(S);

    // // create the constraint: 
    // am_Constraint *c1 = am_newconstraint(S, AM_REQUIRED);
    // am_Constraint *c2 = am_newconstraint(S, AM_REQUIRED);

    // // c1: m is in middle of l and r:
    // //     i.e. m = (l + r) / 2, or 2*m = l + r
    // am_addterm(c1, m, 2.f);
    // am_setrelation(c1, AM_EQUAL);
    // am_addterm(c1, l, 1.f);
    // am_addterm(c1, r, 1.f);
    // // apply c1
    // am_add(c1);

    // // c2: r - l >= 100
    // am_addterm(c2, r, 1.f);
    // am_addterm(c2, l, -1.f);
    // am_setrelation(c2, AM_GREATEQUAL);
    // am_addconstant(c2, 100.f);
    // // apply c2
    // am_add(c2);

    // test anchors
    add_anchor( 50.0f, 50.0f );
    add_anchor( 450.0f, 50.0f );
    add_anchor( 450.0f, 450.0f );
    add_anchor( 50.0f, 450.0f );
    g_anchor[2].pinned = true;

    add_wall(0, 1 );
    add_wall(1, 2 );
    add_wall(2, 3 );
    add_wall(3, 0 );

    // add_constraint_len( 0, 1, -1.0f );
    // add_constraint_len( 1, 2, -1.0f );
    // add_constraint_len( 2, 3, -1.0f );
     add_constraint_len( 3, 0, -1.0f );

    // add_constraint_parallel( 0, 1, 3, 2 );
    //add_constraint_parallel( 1, 2, 0, 3 );

    add_constraint_angle( 0, 1, 2, -1.0f );

    int drag_ndx = -1;
    float dd = 0;


     while (!exitWindow)    // Detect window close button or ESC key
    {

        Vector2 mousePos = GetMousePosition();


    	// Update Solver (amoeba)
    	//----------------------------------------------------------------------------------

    	// now we set variable l to 20
	    // am_suggest(l, 20.f );
        // am_suggest(r, mousePos.x );

	    // // and see the value of m and r:
	    // am_updatevars(S);

	    // // r should by 20 + 100 == 120:
	    // // assert(am_value(r) == 120.f);

	    // // and m should in middle of l and r:
	    // // assert(am_value(m) == 70.f);

	    // float val_left = am_value(l);
	    // float val_mid = am_value(m);
	    // float val_right = am_value(r);

        // Update Solver (custom)
    	//----------------------------------------------------------------------------------
        int steps = 100;
        float base_str = 5.00;
        float str = base_str / (float)steps;
        for (int substep = 0; substep < steps; substep++)
        {
            

            for (int i=0; i < nConstraint; i++) {
                if (g_constraint[i].type == ConstraintType_LENGTH) {
                    eval_constrain_len( g_constraint[i], str );    
                } else if (g_constraint[i].type == ConstraintType_PARALLEL) {
                    eval_constrain_parallel( g_constraint[i], str );    
                } else if (g_constraint[i].type == ConstraintType_ANGLE) {
                    eval_constrain_angle( g_constraint[i], str );    
                }
                
            }

            if (g_pinsActive)
            {
                for (int j=0; j < nAnchor; j++) {
                    if (g_anchor[j].pinned) {
                        g_anchor[j].p = g_anchor[j].p_orig;
                    }
                }
            }

        }
        //exit(1);

        // DBG DRAW
        for (int i=0; i < nConstraint; i++) {
            if (g_constraint[i].type == ConstraintType_PARALLEL) {
                draw_constrain_parallel( g_constraint[i] );    
            }        
        }
     

        // Update GUI
        //----------------------------------------------------------------------------------
        exitWindow = WindowShouldClose();

        // Drag anchors
        if (IsMouseButtonPressed( MOUSE_LEFT_BUTTON)) {
            drag_ndx = -1;
            for (int i=0; i < nAnchor; i++) {
                Anchor *a = g_anchor + i;

                // Save mousedown pos
                a->p_dragstart = a->p;

                float d = Vector2DistanceSqr( a->p, mousePos );
                if ( d <= 20) {
                    if ((drag_ndx < 0) || (d < dd)) {
                        dd = d;
                        drag_ndx = i;
                    }
                }
            }


            printf("Mouse pressed %d\n", drag_ndx );
        }

        if (IsMouseButtonReleased( MOUSE_LEFT_BUTTON)) {
            drag_ndx = -1;
            printf("Mouse released\n");
        }

        // TODO let solver do this
        if (drag_ndx >= 0) {
            g_anchor[drag_ndx].p = mousePos;
            //g_anchor[drag_ndx].p_dragstart = mousePos;

        }

        BeginDrawing();

        // ClearBackground(GetColor(GuiGetStyle(DEFAULT, BACKGROUND_COLOR)));
        ClearBackground( BLACK );

        // DrawLine(  val_left, 50, val_right, 50, WHITE );
        // DrawCircle( val_mid, 50, 5, WHITE );

        // Draw walls
        for (int i=0; i < nWall; i++) {
            Anchor *a = g_anchor + g_wall[i].a;
            Anchor *b = g_anchor + g_wall[i].b;

            DrawSplineSegmentLinear( a->p, b->p, 4.0, DARKGREEN );
        }

        // Draw anchors
        char buff[10] = "*";

        for (int i=0; i < nAnchor; i++) {
            Anchor *a = g_anchor + i;
            Color c = (a->pinned && g_pinsActive) ? RED : GREEN;

            DrawCircle( a->p.x, a->p.y, 10, c );
            buff[0] = 'A' + i;
            DrawText( buff, a->p.x-6, a->p.y-8, 18, BLACK );
        }

        EndDrawing();

        if (IsKeyReleased( KEY_Z )) {
            for (int i=0; i < nAnchor; i++) {
                g_anchor[i].p = g_anchor[i].p_orig;
            }
        }

        if (IsKeyReleased( KEY_X )) {
            g_pinsActive = !g_pinsActive;
        }

        if (IsKeyReleased( KEY_P )) {
            dbgParrActive = !dbgParrActive;
            printf("Parallel Active: %s\n", dbgParrActive?"TRUE":"False");
        }

        if (IsKeyReleased( KEY_O )) {
            dbgAngleActive = !dbgAngleActive;
            printf("Angle Active: %s\n", dbgAngleActive?"TRUE":"False");
        }

    }


 	CloseWindow();
// 	am_delsolver(S);
}
